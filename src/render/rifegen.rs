//! RIFE frame generation (experimental, `rife` feature). Unlike the optical-flow
//! path, inference takes most of a source frame and runs on the CPU side of
//! ONNX Runtime, so frames are shown from a small ring at a steady delay:
//!
//! 1. Each new source frame is copied into `ring[slot]`, box-filtered to the
//!    model's resolution and copied to a staging buffer, which goes to the worker
//!    (crate::rife) after submit.
//! 2. The worker reads it back, runs RIFE on the newest consecutive pair and wakes
//!    the loop; flow + mask are uploaded to `flow[key & 1]`.
//! 3. A schedule decides what's on screen: frame N's pair starts `delay` after N
//!    arrived; the in-between frames are spaced 1/factor of a source interval apart
//!    and N itself follows them. If inference is late, its frames are skipped and
//!    the real frame still shows on time.

use super::{INTERMEDIATE, SourceBinding, fullscreen_pipeline, tex_entry, ubo_entry};
use crate::rife::{FrameMsg, Out, Worker};
use crate::settings::Settings;
use bytemuck::{Pod, Zeroable};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const RING: usize = 4;
const MAX_GAP: f64 = 0.1;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PrepParams {
    out_size: [f32; 2],
    linear: u32,
    _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WarpParams {
    size: [f32; 2],
    fsize: [f32; 2],
    t: f32,
    _pad: [f32; 3],
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Show {
    Real(usize),
    Gen { older: usize, newer: usize, set: usize, t: f32 },
}

#[derive(Clone, Copy, Default)]
struct SlotMeta {
    key: u64,
    valid: bool,
    arrival: Option<Instant>,
}

struct Targets {
    size: [u32; 2],
    fsize: [u32; 2],
    src_generation: u64,
    from_sharp: bool,
    ring: [wgpu::Texture; RING],
    ring_view: [wgpu::TextureView; RING],
    ring_scale_bg: [wgpu::BindGroup; RING],
    prep_bg: [wgpu::BindGroup; RING],
    prep_view: wgpu::TextureView,
    prep_tex: wgpu::Texture,
    staging: [wgpu::Buffer; RING],
    /// Set while the worker owns `staging[i]` (mapped / about to be).
    staging_busy: [Arc<AtomicBool>; RING],
    row_bytes: u32,
    shown_view: wgpu::TextureView,
    shown_scale_bg: wgpu::BindGroup,
    flow: [Option<(wgpu::Texture, wgpu::TextureView, wgpu::Texture, wgpu::TextureView, [u32; 2])>; 2],
    next_slot: usize,
    meta: [SlotMeta; RING],
    id: u64,
}

pub struct RifeGen {
    prep_bgl: wgpu::BindGroupLayout,
    prep_pipeline: wgpu::RenderPipeline,
    prep_ubo: wgpu::Buffer,
    warp_bgl: wgpu::BindGroupLayout,
    warp_pipeline: wgpu::RenderPipeline,
    warp_ubo: wgpu::Buffer,
    sampler: wgpu::Sampler,
    worker: Option<Result<Worker, String>>,
    t: Option<Targets>,
    next_id: u64,
    last_seq: u64,
    prev_arrival: Option<Instant>,
    interval: f64,
    /// Smoothed arrival → flow-ready latency and inference time, seconds / ms.
    latency: f64,
    infer_ms: f64,
    schedule: VecDeque<(Instant, Show)>,
    current: Option<Show>,
    display_counter: u64,
    warp_pending: Option<Show>,
    /// Staged by `encode` for `after_submit`.
    pending_map: Option<usize>,
    factor: u32,
    reason: Option<String>,
    gens_shown: u64,
    gens_late: u64,
}

impl RifeGen {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = |bgl: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(bgl)],
                immediate_size: 0,
            })
        };
        let filt = wgpu::TextureSampleType::Float { filterable: true };
        let nf = wgpu::TextureSampleType::Float { filterable: false };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let prep_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rife_prep"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/rife_prep.wgsl").into()),
        });
        let prep_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rife_prep_bgl"),
            entries: &[ubo_entry(0), tex_entry(1, filt), sampler_entry(2)],
        });
        let prep_pipeline = fullscreen_pipeline(
            device,
            "rife_prep",
            &prep_module,
            &layout(&prep_bgl),
            "fs_main",
            wgpu::TextureFormat::Rgba32Float,
        );
        let warp_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rife_warp"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/rife_warp.wgsl").into()),
        });
        let warp_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rife_warp_bgl"),
            entries: &[
                ubo_entry(0),
                tex_entry(1, filt),
                tex_entry(2, filt),
                tex_entry(3, nf),
                tex_entry(4, nf),
                sampler_entry(5),
            ],
        });
        let warp_pipeline = fullscreen_pipeline(
            device,
            "rife_warp",
            &warp_module,
            &layout(&warp_bgl),
            "fs_main",
            INTERMEDIATE,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rife"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            prep_bgl,
            prep_pipeline,
            prep_ubo: super::ubo(device, "rife_prep", std::mem::size_of::<PrepParams>()),
            warp_bgl,
            warp_pipeline,
            warp_ubo: super::ubo(device, "rife_warp", std::mem::size_of::<WarpParams>()),
            sampler,
            worker: None,
            t: None,
            next_id: 1 << 40,
            last_seq: u64::MAX,
            prev_arrival: None,
            interval: 1.0 / 60.0,
            latency: 0.020,
            infer_ms: 0.0,
            schedule: VecDeque::new(),
            current: None,
            display_counter: 0,
            warp_pending: None,
            pending_map: None,
            factor: 2,
            reason: None,
            gens_shown: 0,
            gens_late: 0,
        }
    }

    pub fn reset(&mut self) {
        self.t = None;
        self.last_seq = u64::MAX;
        self.prev_arrival = None;
        self.schedule.clear();
        self.current = None;
        self.warp_pending = None;
        self.pending_map = None;
        // The worker may still hold a staging buffer of the old targets; dropping
        // them is fine (wgpu keeps the buffer alive), and its result is ignored.
    }

    /// Frame on screen while RIFE is active: scale-pass group 0, view, upscaler id.
    pub fn showing(&self) -> Option<(&wgpu::BindGroup, &wgpu::TextureView, u64)> {
        let t = self.t.as_ref()?;
        match self.current? {
            Show::Real(s) => Some((&t.ring_scale_bg[s], &t.ring_view[s], t.id + s as u64)),
            Show::Gen { .. } => Some((&t.shown_scale_bg, &t.shown_view, t.id + RING as u64)),
        }
    }

    /// Changes whenever a different frame goes on screen.
    pub fn display_key(&self) -> u64 {
        (1 << 63) | self.display_counter
    }

    pub fn due(&self) -> Option<Instant> {
        self.schedule.front().map(|(t, _)| *t)
    }

    pub fn status(&self, refresh_hz: Option<f64>) -> Option<(String, bool)> {
        if let Some(r) = &self.reason {
            return Some((r.clone(), false));
        }
        let w = match &self.worker {
            Some(Ok(w)) => w,
            Some(Err(e)) => return Some((e.clone(), false)),
            None => return None,
        };
        let t = self.t.as_ref()?;
        let fps = 1.0 / self.interval.max(1e-3);
        let n = self.factor as f64;
        let out = fps * n;
        let delay = (self.delay() + self.interval * (n - 1.0) / n) * 1000.0;
        let late = if self.gens_shown + self.gens_late > 0 {
            100.0 * self.gens_late as f64 / (self.gens_shown + self.gens_late) as f64
        } else {
            0.0
        };
        let mut text = format!(
            "{} · {fps:.0} → {out:.0} fps ({n}×) · model {}×{} · {:.1} ms · +{delay:.0} ms",
            w.model.trim_end_matches(".onnx").trim_end_matches("_flow"),
            t.fsize[0],
            t.fsize[1],
            self.infer_ms,
        );
        if late > 0.5 {
            text += &format!(" · {late:.0}% late");
        }
        let over = refresh_hz.filter(|hz| out > hz * 1.03);
        if let Some(hz) = over {
            text += &format!(" · display is {hz:.0} Hz");
        }
        Some((text, over.is_none() && late < 10.0))
    }

    fn delay(&self) -> f64 {
        (self.latency + 0.003).clamp(0.004, self.interval * 2.5)
    }

    fn factor_for(&self, s: &Settings, refresh_hz: Option<f64>) -> u32 {
        match s.frame_gen_factor {
            f @ 2..=4 => f as u32,
            _ => {
                let fps = 1.0 / self.interval.max(1e-3);
                let fit = refresh_hz.map_or(2.0, |hz| (hz / fps + 0.05).floor());
                (fit as u32).clamp(2, 4)
            }
        }
    }

    fn push_schedule(&mut self, at: Instant, show: Show) {
        let i = self.schedule.partition_point(|(t, _)| *t <= at);
        self.schedule.insert(i, (at, show));
    }

    /// Per render. Returns true when the caller must submit `encoder` and then call
    /// `after_submit` (a readback to map).
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        src: Option<&SourceBinding>,
        frame_seq: u64,
        arrival: Option<Instant>,
        s: &Settings,
        scale: (&wgpu::BindGroupLayout, &wgpu::Buffer, &wgpu::Sampler),
        refresh_hz: Option<f64>,
    ) -> bool {
        self.pending_map = None;
        if self.worker.is_none() {
            self.worker = Some(Worker::spawn(device.clone()));
            if let Some(Err(e)) = &self.worker {
                log::warn!("RIFE unavailable: {e}");
            }
        }
        if !matches!(self.worker, Some(Ok(_))) {
            return false;
        }
        let now = Instant::now();

        // 1. What's on screen now.
        let mut changed = None;
        while self.schedule.front().is_some_and(|(t, _)| *t <= now) {
            let (_, show) = self.schedule.pop_front().expect("front");
            if let Some(Show::Gen { .. }) = changed {
                self.gens_late += 1; // superseded before it was shown
            }
            changed = Some(show);
        }
        if let Some(show) = changed {
            if self.current != Some(show) {
                self.current = Some(show);
                self.display_counter += 1;
                if let Show::Gen { .. } = show {
                    self.gens_shown += 1;
                    self.warp_pending = Some(show);
                }
            }
        }

        // 3. Finished inference → flow textures + in-between frames on the schedule.
        let results: Vec<Result<Out, String>> = match &self.worker {
            Some(Ok(w)) => std::iter::from_fn(|| w.try_recv()).collect(),
            _ => Vec::new(),
        };
        for r in results {
            match r {
                Ok(out) => self.accept(device, queue, out, now),
                Err(e) => {
                    log::warn!("RIFE: {e}");
                    self.reason = Some(format!("RIFE: {e}"));
                }
            }
        }

        // 5. A new source frame.
        let Some(src) = src else {
            self.current = None;
            return false;
        };
        if frame_seq == self.last_seq || frame_seq == u64::MAX {
            return false;
        }
        self.last_seq = frame_seq;
        let size = [src.width.max(1), src.height.max(1)];
        let from_sharp = s.sharpen > 0.001;
        if self.t.as_ref().is_none_or(|t| {
            t.src_generation != src.generation || t.size != size || t.from_sharp != from_sharp
        }) {
            self.reset();
            self.last_seq = frame_seq;
            self.t = Some(self.build_targets(device, src, size, from_sharp, scale));
        }
        let arrival = arrival.unwrap_or(now);
        let gap = self
            .prev_arrival
            .map(|p| arrival.saturating_duration_since(p).as_secs_f64());
        if let Some(g) = gap.filter(|g| *g > 0.0 && *g < MAX_GAP) {
            self.interval = self.interval * 0.9 + g * 0.1;
        }
        self.prev_arrival = Some(arrival);
        self.factor = self.factor_for(s, refresh_hz);

        let delay = self.delay();
        let interval = self.interval;
        let factor = self.factor;
        let t = self.t.as_mut().expect("targets");
        let slot = t.next_slot;
        t.next_slot = (slot + 1) % RING;
        // This slot is about to be overwritten: drop anything that still shows it.
        self.schedule.retain(|(_, sh)| match sh {
            Show::Real(s) => *s != slot,
            Show::Gen { older, newer, .. } => *older != slot && *newer != slot,
        });
        // The worker must be done with its staging buffer before the copy; if it's
        // that far behind, this frame just isn't interpolated.
        let read_back = !t.staging_busy[slot].load(Ordering::Acquire);
        t.meta[slot] = SlotMeta {
            key: frame_seq,
            valid: true,
            arrival: Some(arrival),
        };
        let src_tex = if from_sharp { &src.sharp_tex } else { &src.rgb_tex };
        encoder.copy_texture_to_texture(
            src_tex.as_image_copy(),
            t.ring[slot].as_image_copy(),
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        queue.write_buffer(
            &self.prep_ubo,
            0,
            bytemuck::bytes_of(&PrepParams {
                out_size: [t.fsize[0] as f32, t.fsize[1] as f32],
                linear: s.linear_light as u32,
                _pad: 0,
            }),
        );
        {
            let mut pass = super::begin_pass(encoder, "rife_prep", &t.prep_view);
            pass.set_pipeline(&self.prep_pipeline);
            pass.set_bind_group(0, &t.prep_bg[slot], &[]);
            pass.draw(0..3, 0..1);
        }
        if read_back { encoder.copy_texture_to_buffer(
            t.prep_tex.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &t.staging[slot],
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(t.row_bytes),
                    rows_per_image: Some(t.fsize[1]),
                },
            },
            wgpu::Extent3d {
                width: t.fsize[0],
                height: t.fsize[1],
                depth_or_array_layers: 1,
            },
        ); }
        // The real frame goes on screen after its pair's in-between frames.
        let start = arrival + Duration::from_secs_f64(delay);
        let real_at = start + Duration::from_secs_f64(interval * (factor - 1) as f64 / factor as f64);
        self.push_schedule(real_at.max(now), Show::Real(slot));
        if self.current.is_none() {
            // Nothing on screen yet (first frame): show it right away.
            self.current = Some(Show::Real(slot));
            self.display_counter += 1;
        }
        self.pending_map = read_back.then_some(slot);
        true
    }

    /// Map the readback staged by `encode` (after its encoder was submitted).
    pub fn after_submit(&mut self) {
        let Some(slot) = self.pending_map.take() else { return };
        let (Some(t), Some(Ok(w))) = (&self.t, &self.worker) else { return };
        t.staging_busy[slot].store(true, Ordering::Release);
        w.submit(FrameMsg {
            key: t.meta[slot].key,
            buffer: t.staging[slot].clone(),
            row_bytes: t.row_bytes,
            w: t.fsize[0],
            h: t.fsize[1],
            in_use: t.staging_busy[slot].clone(),
        });
    }

    fn accept(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, out: Out, now: Instant) {
        self.infer_ms = if self.infer_ms == 0.0 { out.ms } else { self.infer_ms * 0.9 + out.ms * 0.1 };
        let Some(t) = self.t.as_mut() else { return };
        let find = |key: u64| (0..RING).find(|&i| t.meta[i].valid && t.meta[i].key == key);
        let (Some(newer), Some(older)) = (find(out.key), find(out.key.wrapping_sub(1))) else {
            return; // overwritten meanwhile
        };
        let Some(arrival) = t.meta[newer].arrival else { return };
        let lat = out.done.saturating_duration_since(arrival).as_secs_f64();
        self.latency = self.latency * 0.9 + lat * 0.1;
        let set = (out.key & 1) as usize;
        let dims = [out.fw, out.fh];
        if t.flow[set].as_ref().is_none_or(|f| f.4 != dims) {
            let mk = |label, format| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: dims[0],
                        height: dims[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                })
            };
            let flow = mk("rife_flow", wgpu::TextureFormat::Rgba32Float);
            let mask = mk("rife_mask", wgpu::TextureFormat::R32Float);
            let fv = flow.create_view(&Default::default());
            let mv = mask.create_view(&Default::default());
            t.flow[set] = Some((flow, fv, mask, mv, dims));
        }
        let f = t.flow[set].as_ref().expect("flow");
        let extent = wgpu::Extent3d {
            width: dims[0],
            height: dims[1],
            depth_or_array_layers: 1,
        };
        queue.write_texture(
            f.0.as_image_copy(),
            bytemuck::cast_slice(&out.flow),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(dims[0] * 16),
                rows_per_image: Some(dims[1]),
            },
            extent,
        );
        queue.write_texture(
            f.2.as_image_copy(),
            bytemuck::cast_slice(&out.mask),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(dims[0] * 4),
                rows_per_image: Some(dims[1]),
            },
            extent,
        );
        let start = arrival + Duration::from_secs_f64(self.delay());
        let n = self.factor;
        for k in 1..n {
            let at = start + Duration::from_secs_f64(self.interval * (k - 1) as f64 / n as f64);
            let show = Show::Gen {
                older,
                newer,
                set,
                t: k as f32 / n as f32,
            };
            if at + Duration::from_secs_f64(self.interval / n as f64) < now {
                self.gens_late += 1; // its slot has already passed
                continue;
            }
            self.push_schedule(at.max(now), show);
        }
    }

    /// Render the in-between frame that just came due.
    pub fn encode_warp(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) {
        let Some(Show::Gen { older, newer, set, t: tt }) = self.warp_pending.take() else {
            return;
        };
        let Some(t) = &self.t else { return };
        let Some(f) = &t.flow[set] else { return };
        queue.write_buffer(
            &self.warp_ubo,
            0,
            bytemuck::bytes_of(&WarpParams {
                size: [t.size[0] as f32, t.size[1] as f32],
                fsize: [t.fsize[0] as f32, t.fsize[1] as f32],
                t: tt,
                _pad: [0.0; 3],
            }),
        );
        let tv = |binding, v| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(v),
        };
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rife_warp"),
            layout: &self.warp_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.warp_ubo.as_entire_binding(),
                },
                tv(1, &t.ring_view[older]),
                tv(2, &t.ring_view[newer]),
                tv(3, &f.1),
                tv(4, &f.3),
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut pass = super::begin_pass(encoder, "rife_warp", &t.shown_view);
        pass.set_pipeline(&self.warp_pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..3, 0..1);
    }

    fn build_targets(
        &mut self,
        device: &wgpu::Device,
        src: &SourceBinding,
        size: [u32; 2],
        from_sharp: bool,
        (scale_bgl, scale_ubo, scale_sampler): (&wgpu::BindGroupLayout, &wgpu::Buffer, &wgpu::Sampler),
    ) -> Targets {
        let _ = src;
        // Model resolution (like LSFG's flow scale): 360 lines by default, which keeps
        // inference well inside a 60 fps frame; MAXCAP_RIFE_HEIGHT overrides it.
        let want = std::env::var("MAXCAP_RIFE_HEIGHT")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(360)
            .clamp(128, size[1]);
        let fh = want;
        let fw = ((size[0] as f64 * fh as f64 / size[1] as f64).round() as u32).max(1);
        let fsize = [fw, fh];
        let mk = |label: &str, w: u32, h: u32, format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        use wgpu::TextureUsages as U;
        let ring: [wgpu::Texture; RING] = std::array::from_fn(|i| {
            mk(&format!("rife_ring{i}"), size[0], size[1], INTERMEDIATE, U::COPY_DST | U::TEXTURE_BINDING)
        });
        let ring_view: [wgpu::TextureView; RING] = std::array::from_fn(|i| ring[i].create_view(&Default::default()));
        let shown = mk("rife_shown", size[0], size[1], INTERMEDIATE, U::RENDER_ATTACHMENT | U::TEXTURE_BINDING);
        let shown_view = shown.create_view(&Default::default());
        let prep_tex = mk(
            "rife_prep",
            fsize[0],
            fsize[1],
            wgpu::TextureFormat::Rgba32Float,
            U::RENDER_ATTACHMENT | U::COPY_SRC,
        );
        let prep_view = prep_tex.create_view(&Default::default());
        let row_bytes = (fsize[0] * 16).div_ceil(256) * 256;
        let staging: [wgpu::Buffer; RING] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("rife_staging{i}")),
                size: (row_bytes * fsize[1]) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let scale_bg = |v: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scale_bg_rife"),
                layout: scale_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: scale_ubo.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(v),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(scale_sampler),
                    },
                ],
            })
        };
        let ring_scale_bg: [wgpu::BindGroup; RING] = std::array::from_fn(|i| scale_bg(&ring_view[i]));
        let shown_scale_bg = scale_bg(&shown_view);
        let prep_bg: [wgpu::BindGroup; RING] = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("rife_prep"),
                layout: &self.prep_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.prep_ubo.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&ring_view[i]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            })
        });
        let id = self.next_id;
        self.next_id += 16;
        log::info!(
            "RIFE frame generation: {}x{} source, model at {}x{}",
            size[0],
            size[1],
            fsize[0],
            fsize[1]
        );
        Targets {
            size,
            fsize,
            src_generation: src.generation,
            from_sharp,
            ring,
            ring_view,
            ring_scale_bg,
            prep_bg,
            prep_view,
            prep_tex,
            staging,
            staging_busy: std::array::from_fn(|_| Arc::new(AtomicBool::new(false))),
            row_bytes,
            shown_view,
            shown_scale_bg,
            flow: [None, None],
            next_slot: 0,
            meta: [SlotMeta::default(); RING],
            id,
        }
    }
}
