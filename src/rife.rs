//! RIFE frame interpolation (experimental, `rife` feature): runs a trimmed RIFE
//! v4 ONNX model on DirectML on a worker thread. The model takes two frames at
//! reduced resolution plus t and returns the intermediate flow (to each frame) and
//! the blend mask; render/rifegen.rs does the full-resolution warp itself.
//!
//! The worker owns the whole CPU side so nothing waits on a redraw: it waits for
//! each frame's readback, keeps the previous frame, runs the model on the newest
//! consecutive pair and wakes the event loop when the result is ready.
//!
//! Needs onnxruntime.dll + DirectML.dll (ONNX Runtime 1.24, DirectML build) and a
//! model (`rife_v4.26_flow.onnx`, or MAXCAP_RIFE_MODEL=<file name>) next to the exe.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

/// Wakes the event loop (set by main).
pub static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// A frame's staging buffer, already submitted for copy (RGBA f32 rows).
pub struct FrameMsg {
    pub key: u64,
    pub buffer: wgpu::Buffer,
    pub row_bytes: u32,
    pub w: u32,
    pub h: u32,
    /// Cleared once the buffer is unmapped again (it may be reused after that).
    pub in_use: Arc<AtomicBool>,
}

pub struct Out {
    /// Source sequence of the newer frame of the pair.
    pub key: u64,
    /// RGBA-interleaved flow: xy = offset to the older frame, zw = to the newer one,
    /// in model-input pixels. `fw * fh` texels (the model pads the input size).
    pub flow: Vec<f32>,
    /// Weight of the older frame, `fw * fh`.
    pub mask: Vec<f32>,
    pub fw: u32,
    pub fh: u32,
    pub ms: f64,
    /// When the result was ready.
    pub done: Instant,
}

pub struct Worker {
    tx: Sender<FrameMsg>,
    rx: Receiver<Result<Out, String>>,
    pub model: String,
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Worker {
    pub fn spawn(device: wgpu::Device) -> Result<Self, String> {
        let dir = exe_dir();
        let model = std::env::var("MAXCAP_RIFE_MODEL").unwrap_or_else(|_| "rife_v4.26_flow.onnx".into());
        let model_path = dir.join(&model);
        if !model_path.exists() {
            return Err(format!("RIFE model {model} not found next to the exe"));
        }
        let dll = dir.join("onnxruntime.dll");
        if !dll.exists() {
            return Err("onnxruntime.dll not found next to the exe".into());
        }
        let ok = ort::init_from(&dll)
            .map_err(|e| format!("ONNX Runtime: {e}"))?
            .with_name("maxcap")
            .commit();
        if !ok {
            log::info!("ONNX Runtime environment already initialised");
        }
        let (tx, frames) = channel::<FrameMsg>();
        let (done, rx) = channel::<Result<Out, String>>();
        std::thread::Builder::new()
            .name("rife".into())
            .spawn(move || {
                let mut session: Option<(usize, usize, ort::session::Session)> = None;
                // (key, planar RGB, w, h)
                let mut prev: Option<(u64, Vec<f32>, usize, usize)> = None;
                let mut cur: Option<(u64, Vec<f32>, usize, usize)> = None;
                while let Ok(first) = frames.recv() {
                    // Read every queued frame (cheap), but only run the model on the
                    // newest pair.
                    let mut batch = vec![first];
                    while let Ok(m) = frames.try_recv() {
                        batch.push(m);
                    }
                    for m in batch {
                        let data = read(&device, &m);
                        m.in_use.store(false, Ordering::Release);
                        let Some(data) = data else { continue };
                        prev = cur.take();
                        cur = Some((m.key, data, m.w as usize, m.h as usize));
                    }
                    let (Some((pk, pd, pw, ph)), Some((ck, cd, w, h))) = (&prev, &cur) else {
                        continue;
                    };
                    if *ck != pk.wrapping_add(1) || (pw, ph) != (w, h) {
                        continue;
                    }
                    let r = run(&mut session, &model_path, *ck, pd, cd, (*w, *h));
                    if done.send(r).is_err() {
                        break;
                    }
                    if let Some(wake) = WAKE.get() {
                        wake();
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        log::info!("RIFE worker ready ({model})");
        Ok(Self { tx, rx, model })
    }

    pub fn submit(&self, frame: FrameMsg) {
        let _ = self.tx.send(frame);
    }

    pub fn try_recv(&self) -> Option<Result<Out, String>> {
        self.rx.try_recv().ok()
    }
}

/// Wait for the buffer's copy, then convert its RGBA rows to planar RGB.
fn read(device: &wgpu::Device, m: &FrameMsg) -> Option<Vec<f32>> {
    let (tx, rx) = channel();
    m.buffer.map_async(wgpu::MapMode::Read, .., move |r| {
        let _ = tx.send(r.is_ok());
    });
    let deadline = Instant::now() + Duration::from_millis(500);
    let ok = loop {
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_millis(50)),
        });
        match rx.try_recv() {
            Ok(ok) => break ok,
            Err(_) if Instant::now() > deadline => return None,
            Err(_) => {}
        }
    };
    if !ok {
        return None;
    }
    let (w, h) = (m.w as usize, m.h as usize);
    let n = w * h;
    let mut planar = vec![0f32; n * 3];
    if let Ok(view) = m.buffer.slice(..).get_mapped_range() {
        let floats: &[f32] = bytemuck::cast_slice(&view);
        let stride = m.row_bytes as usize / 4;
        for y in 0..h {
            let row = &floats[y * stride..y * stride + w * 4];
            for x in 0..w {
                let i = y * w + x;
                planar[i] = row[x * 4];
                planar[n + i] = row[x * 4 + 1];
                planar[2 * n + i] = row[x * 4 + 2];
            }
        }
    }
    m.buffer.unmap();
    Some(planar)
}

fn open(path: &std::path::Path, w: usize, h: usize) -> Result<ort::session::Session, String> {
    let e = |e: ort::Error<ort::session::builder::SessionBuilder>| e.to_string();
    let s = ort::session::Session::builder()
        .map_err(|e| e.to_string())?
        .with_memory_pattern(false)
        .map_err(e)?
        .with_parallel_execution(false)
        .map_err(e)?
        .with_dimension_override("batch", 1)
        .map_err(e)?
        .with_dimension_override("height", h as i64)
        .map_err(e)?
        .with_dimension_override("width", w as i64)
        .map_err(e)?
        .with_execution_providers([ort::ep::DirectML::default().build().error_on_failure()])
        .map_err(e)?
        .commit_from_file(path)
        .map_err(|e| e.to_string())?;
    log::info!("RIFE session {w}x{h} on DirectML");
    Ok(s)
}

fn run(
    session: &mut Option<(usize, usize, ort::session::Session)>,
    path: &std::path::Path,
    key: u64,
    prev: &[f32],
    next: &[f32],
    (w, h): (usize, usize),
) -> Result<Out, String> {
    if session.as_ref().is_none_or(|(sw, sh, _)| (*sw, *sh) != (w, h)) {
        *session = None;
        *session = Some((w, h, open(path, w, h)?));
    }
    let (_, _, s) = session.as_mut().expect("session");
    let n = w * h;
    let mut input = Vec::with_capacity(7 * n);
    input.extend_from_slice(prev);
    input.extend_from_slice(next);
    input.resize(7 * n, 0.5); // t plane
    let t0 = Instant::now();
    let tensor = ort::value::Tensor::from_array(([1usize, 7, h, w], input)).map_err(|e| e.to_string())?;
    let outputs = s
        .run(ort::inputs!["input" => tensor])
        .map_err(|e| e.to_string())?;
    let (shape, flow) = outputs["flow"]
        .try_extract_tensor::<f32>()
        .map_err(|e| e.to_string())?;
    let (fh, fw) = (shape[2] as usize, shape[3] as usize);
    let m = fw * fh;
    let mut rgba = vec![0f32; m * 4];
    for i in 0..m {
        rgba[i * 4] = flow[i];
        rgba[i * 4 + 1] = flow[m + i];
        rgba[i * 4 + 2] = flow[2 * m + i];
        rgba[i * 4 + 3] = flow[3 * m + i];
    }
    let (_, mask) = outputs["mask"]
        .try_extract_tensor::<f32>()
        .map_err(|e| e.to_string())?;
    Ok(Out {
        key,
        flow: rgba,
        mask: mask[..m].to_vec(),
        fw: fw as u32,
        fh: fh as u32,
        ms: t0.elapsed().as_secs_f64() * 1000.0,
        done: Instant::now(),
    })
}
