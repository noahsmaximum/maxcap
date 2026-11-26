//! Persistent user settings (JSON in %APPDATA%\MaxCap\settings.json).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorMatrix {
    Auto,
    Bt601,
    Bt709,
    Bt2020,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorRange {
    Auto,
    Limited,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChromaFilter {
    Nearest,
    Linear,
    Cubic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChromaSiting {
    /// Use what the device/format reports.
    Auto,
    /// Co-sited with the left luma sample (MPEG-2 / HDMI 4:2:2 default).
    Left,
    /// Centered between luma samples (MPEG-1 / JPEG).
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleFilter {
    Nearest,
    Bilinear,
    SharpBilinear,
    CatmullRom,
    Mitchell,
    Lanczos2,
    Lanczos3,
}

impl ScaleFilter {
    pub const ALL: [ScaleFilter; 7] = [
        ScaleFilter::Nearest,
        ScaleFilter::Bilinear,
        ScaleFilter::SharpBilinear,
        ScaleFilter::CatmullRom,
        ScaleFilter::Mitchell,
        ScaleFilter::Lanczos2,
        ScaleFilter::Lanczos3,
    ];
    pub fn label(self) -> &'static str {
        match self {
            ScaleFilter::Nearest => "Nearest (pixel exact)",
            ScaleFilter::Bilinear => "Bilinear",
            ScaleFilter::SharpBilinear => "Sharp bilinear (pixel art)",
            ScaleFilter::CatmullRom => "Bicubic Catmull-Rom",
            ScaleFilter::Mitchell => "Bicubic Mitchell",
            ScaleFilter::Lanczos2 => "Lanczos 2",
            ScaleFilter::Lanczos3 => "Lanczos 3",
        }
    }
    pub fn id(self) -> u32 {
        self as u32
    }
}

/// AI upscaling of the picture before it's fitted to the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Upscaler {
    Off,
    /// NVIDIA RTX Video Super Resolution (Windows, GeForce RTX).
    RtxVsr,
}

/// Resolution the AI upscaler renders at; the scaler fits that to the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpscaleTarget {
    /// Exactly the on-screen video size (sharpest; follows the window).
    Window,
    P1440,
    P2160,
    X2,
    X3,
    X4,
}

impl UpscaleTarget {
    pub const ALL: [UpscaleTarget; 6] = [
        UpscaleTarget::Window,
        UpscaleTarget::P1440,
        UpscaleTarget::P2160,
        UpscaleTarget::X2,
        UpscaleTarget::X3,
        UpscaleTarget::X4,
    ];
    pub fn label(self) -> &'static str {
        match self {
            UpscaleTarget::Window => "Match window",
            UpscaleTarget::P1440 => "1440p",
            UpscaleTarget::P2160 => "2160p (4K)",
            UpscaleTarget::X2 => "2x source",
            UpscaleTarget::X3 => "3x source",
            UpscaleTarget::X4 => "4x source",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectMode {
    /// Fit inside the window, keep source aspect.
    Fit,
    /// Fill the window, crop overflow.
    Fill,
    /// Stretch to window.
    Stretch,
    /// Largest integer multiple that fits.
    Integer,
    /// 1:1 pixels, centered.
    Native,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectOverride {
    Source,
    R16_9,
    R4_3,
    R21_9,
    R16_10,
    R1_1,
}

impl AspectOverride {
    pub fn ratio(self) -> Option<f32> {
        match self {
            AspectOverride::Source => None,
            AspectOverride::R16_9 => Some(16.0 / 9.0),
            AspectOverride::R4_3 => Some(4.0 / 3.0),
            AspectOverride::R21_9 => Some(64.0 / 27.0),
            AspectOverride::R16_10 => Some(16.0 / 10.0),
            AspectOverride::R1_1 => Some(1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresentMode {
    /// Tearing allowed, lowest latency (best in fullscreen).
    Immediate,
    /// Latest-frame-wins without tearing.
    Mailbox,
    /// Classic vsync.
    Fifo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputDepth {
    Bits8,
    Bits10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundMode {
    Color,
    Image,
    /// Fill the bars from the live feed itself (glow / streaks / mirror).
    Ambient,
}

/// How the Ambient background extends the video into the bars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmbientStyle {
    /// The whole frame, heavily blurred and scaled up behind the video.
    Glow,
    /// Edge pixels pulled outward forever, blurring and flowing as they go.
    Streaks,
    /// The picture reflected across each edge, softening with distance.
    Mirror,
}

impl AmbientStyle {
    pub fn id(self) -> u32 {
        self as u32
    }
}

/// How a background image covers the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFit {
    /// Fill the window, cropping overflow.
    Cover,
    /// Whole image visible, colour around it.
    Contain,
    Stretch,
    /// 1:1 pixels, centred.
    Center,
    Tile,
}

impl ImageFit {
    pub fn id(self) -> u32 {
        self as u32
    }
}

/// Decoration drawn around the video rectangle; adapts to every size mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameStyle {
    None,
    /// Solid outline in `frame_color`.
    Outline,
    /// OVERWORLD card: bone band, ink rules, hard cobalt shadow.
    PixelCard,
    /// Hard offset shadow only.
    HardShadow,
    /// User PNG, 9-slice scaled around the video.
    Image,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Francais,
}

/// Which lines the stats HUD shows. One flag per line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HudItems {
    pub header: bool,
    pub source: bool,
    pub upscale: bool,
    pub framegen: bool,
    pub colour: bool,
    pub fps: bool,
    pub pacing: bool,
    pub latency: bool,
    pub gpu_cpu: bool,
    pub audio: bool,
    pub output: bool,
    pub graph: bool,
    /// Frame the HUD in the boxed card; off = bare text.
    pub boxed: bool,
}

impl Default for HudItems {
    fn default() -> Self {
        Self::ALL
    }
}

impl HudItems {
    pub const ALL: HudItems = HudItems {
        header: true,
        source: true,
        upscale: true,
        framegen: true,
        colour: true,
        fps: true,
        pacing: true,
        latency: true,
        gpu_cpu: true,
        audio: true,
        output: true,
        graph: true,
        boxed: true,
    };
    const NONE: HudItems = HudItems {
        header: false,
        source: false,
        upscale: false,
        framegen: false,
        colour: false,
        fps: false,
        pacing: false,
        latency: false,
        gpu_cpu: false,
        audio: false,
        output: false,
        graph: false,
        boxed: true,
    };
    pub const PRESETS: [(&'static str, HudItems); 5] = [
        ("Everything", HudItems::ALL),
        ("FPS only", HudItems { fps: true, boxed: false, ..HudItems::NONE }),
        ("FPS + latency", HudItems { fps: true, latency: true, ..HudItems::NONE }),
        (
            "Gaming",
            HudItems { fps: true, pacing: true, latency: true, graph: true, ..HudItems::NONE },
        ),
        (
            "Diagnostics",
            HudItems { header: true, source: true, fps: true, pacing: true, audio: true, ..HudItems::NONE },
        ),
    ];
    /// Name of the matching built-in HUD preset, or "Custom".
    pub fn preset_name(&self) -> &'static str {
        Self::PRESETS.iter().find(|(_, p)| p == self).map_or("Custom", |(n, _)| n)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // --- device ---
    /// Symbolic link of the preferred video device (stable across reboots).
    pub device_link: Option<String>,
    pub device_name: Option<String>,
    /// Preferred format, as "SUBTYPE WxH@num/den". None = best available.
    pub format_key: Option<String>,

    // --- picture ---
    pub matrix: ColorMatrix,
    pub range: ColorRange,
    pub chroma_filter: ChromaFilter,
    pub chroma_siting: ChromaSiting,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub hue: f32,
    pub gamma: f32,
    pub sharpen: f32,

    // --- geometry ---
    pub scale_filter: ScaleFilter,
    pub upscaler: Upscaler,
    pub upscale_target: UpscaleTarget,
    /// RTX VSR quality 1 (low) ..= 4 (ultra).
    pub vsr_quality: u8,
    /// NVIDIA optical-flow frame generation (adds up to a source frame of latency).
    pub frame_gen: bool,
    /// Frames shown per source frame: 2..=4, or 0 = fit the display's refresh rate.
    pub frame_gen_factor: u8,
    /// Artifact guard for generated frames: 0 = off .. 1 = strictest.
    pub frame_gen_guard: f32,
    pub antiring: f32,
    pub linear_light: bool,
    pub aspect_mode: AspectMode,
    pub aspect_override: AspectOverride,
    pub zoom: f32,
    pub crop_l: u32,
    pub crop_r: u32,
    pub crop_t: u32,
    pub crop_b: u32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub rotation: Rotation,
    pub background: [f32; 3],

    // --- background & frame ---
    pub background_mode: BackgroundMode,
    pub background_image: Option<String>,
    pub background_fit: ImageFit,
    /// 0 = image as-is, 1 = black.
    pub background_dim: f32,
    pub ambient_style: AmbientStyle,
    /// Brightness of the ambient fill (0..1.5).
    pub ambient_strength: f32,
    /// How soft it gets (0..1).
    pub ambient_blur: f32,
    /// How far it reaches before fading to the background colour (1 = forever).
    pub ambient_reach: f32,
    /// Flowing motion along the streaks (0 = still).
    pub ambient_flow: f32,
    pub frame_style: FrameStyle,
    pub frame_image: Option<String>,
    /// Corner size of the 9-slice frame image, in image pixels.
    pub frame_slice: u32,
    /// Frame band thickness on screen, in pixels.
    pub frame_thickness: f32,
    /// How far an image frame overlaps the video edge (hides rounded inner corners).
    pub frame_overlap: f32,
    pub frame_color: [f32; 3],
    /// Pixel card colours: band fill, edge lines, hard shadow.
    pub card_fill: [f32; 3],
    pub card_line: [f32; 3],
    pub card_shadow: [f32; 3],
    /// Extra margin between the window edge and the framed video, in pixels.
    pub video_inset: f32,

    // --- display ---
    pub present_mode: PresentMode,
    pub frame_latency: u32,
    pub output_depth: OutputDepth,
    pub dither: bool,
    pub always_on_top: bool,
    pub start_fullscreen: bool,
    pub hide_cursor_secs: f32,
    pub window_size: Option<[u32; 2]>,

    // --- audio ---
    pub audio_enabled: bool,
    pub audio_input: Option<String>,
    pub audio_output: Option<String>,
    pub volume: f32,
    pub muted: bool,
    pub audio_buffer_ms: f32,
    /// Re-interpret MS2109 "96 kHz mono" audio as 48 kHz stereo.
    pub audio_ms2109_fix: bool,
    /// Remove the DC offset some cards' analog audio stage adds.
    pub audio_dc_block: bool,
    /// Fade the card's hiss to silence when nothing is playing.
    pub audio_gate: bool,
    pub audio_gate_db: f32,
    /// Spectral hiss reduction under sound (+~11 ms audio latency).
    pub audio_nr: bool,
    pub audio_nr_strength: f32,

    // --- overlay ---
    pub show_stats: bool,
    pub stats_corner: u8,
    pub hud: HudItems,
    /// HUD top-left as a fraction of the window (None = snap to `stats_corner`).
    pub hud_pos: Option<[f32; 2]>,
    pub hud_opacity: f32,
    /// Window opacity in ghost mode (click-through, borderless, on top).
    pub ghost_opacity: f32,
    /// Capture card name -> preset name applied when that card becomes active.
    pub device_profiles: std::collections::BTreeMap<String, String>,
    pub ui_scale: f32,
    pub language: Language,
    pub screenshot_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            device_link: None,
            device_name: None,
            format_key: None,
            matrix: ColorMatrix::Auto,
            range: ColorRange::Auto,
            chroma_filter: ChromaFilter::Cubic,
            chroma_siting: ChromaSiting::Auto,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue: 0.0,
            gamma: 1.0,
            sharpen: 0.0,
            scale_filter: ScaleFilter::Lanczos3,
            upscaler: Upscaler::Off,
            upscale_target: UpscaleTarget::Window,
            vsr_quality: 4,
            frame_gen: false,
            frame_gen_factor: 2,
            frame_gen_guard: 0.5,
            antiring: 0.6,
            linear_light: false,
            aspect_mode: AspectMode::Fit,
            aspect_override: AspectOverride::Source,
            zoom: 1.0,
            crop_l: 0,
            crop_r: 0,
            crop_t: 0,
            crop_b: 0,
            flip_h: false,
            flip_v: false,
            rotation: Rotation::R0,
            background: [0.0, 0.0, 0.0],
            background_mode: BackgroundMode::Color,
            background_image: None,
            background_fit: ImageFit::Cover,
            background_dim: 0.0,
            ambient_style: AmbientStyle::Glow,
            ambient_strength: 0.85,
            ambient_blur: 0.5,
            ambient_reach: 0.7,
            ambient_flow: 0.35,
            frame_style: FrameStyle::None,
            frame_image: None,
            frame_slice: 32,
            frame_thickness: 24.0,
            frame_overlap: 0.0,
            frame_color: [0.086, 0.075, 0.063],
            card_fill: [1.0, 0.980, 0.937],
            card_line: [0.086, 0.075, 0.063],
            card_shadow: [0.180, 0.365, 0.839],
            video_inset: 0.0,
            present_mode: PresentMode::Immediate,
            frame_latency: 1,
            output_depth: OutputDepth::Bits10,
            dither: true,
            always_on_top: false,
            start_fullscreen: false,
            hide_cursor_secs: 1.5,
            window_size: None,
            audio_enabled: true,
            audio_input: None,
            audio_output: None,
            volume: 1.0,
            muted: false,
            audio_buffer_ms: 30.0,
            audio_ms2109_fix: true,
            audio_dc_block: true,
            audio_gate: true,
            audio_gate_db: -50.0,
            audio_nr: true,
            audio_nr_strength: 0.6,
            show_stats: false,
            stats_corner: 0,
            hud: HudItems::ALL,
            hud_pos: None,
            hud_opacity: 1.0,
            ghost_opacity: 0.6,
            device_profiles: Default::default(),
            ui_scale: 1.0,
            language: Language::English,
            screenshot_dir: None,
        }
    }
}

pub fn config_dir() -> PathBuf {
    // Test hook: isolate settings/presets/images (MAXCAP_CONFIG_DIR=<folder>).
    if let Some(d) = std::env::var_os("MAXCAP_CONFIG_DIR") {
        return PathBuf::from(d);
    }
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("MaxCap");
    static MIGRATE: std::sync::Once = std::sync::Once::new();
    MIGRATE.call_once(|| migrate_legacy_dir(&base.join("capview"), &dir));
    dir
}

/// The app was called "capview" before v0.2: move its settings, presets and imported
/// images into the new folder, and point stored image paths at their new location.
/// Runs once per launch and only when the new folder doesn't exist yet.
fn migrate_legacy_dir(old: &std::path::Path, new: &std::path::Path) {
    if new.exists() || !old.is_dir() {
        return;
    }
    if std::fs::rename(old, new).is_err() {
        // Something holds the old folder open: copy instead, into a staging folder that is
        // renamed into place only when complete, so a failed copy never leaves a half-filled
        // new folder that would block the next launch from retrying.
        let staging = new.with_extension("migrating");
        let _ = std::fs::remove_dir_all(&staging);
        if let Err(e) = copy_dir(old, &staging).and_then(|_| std::fs::rename(&staging, new)) {
            let _ = std::fs::remove_dir_all(&staging);
            log::warn!("couldn't migrate settings from {}: {e}", old.display());
            return;
        }
    }
    log::info!("migrated settings from {} to {}", old.display(), new.display());
    let (old_s, new_s) = (old.to_string_lossy(), new.to_string_lossy());
    let mut files = vec![new.join("settings.json")];
    if let Ok(rd) = std::fs::read_dir(new.join("presets")) {
        files.extend(rd.flatten().map(|e| e.path()));
    }
    for f in files.iter().filter(|f| f.extension().is_some_and(|x| x == "json")) {
        let Ok(text) = std::fs::read_to_string(f) else { continue };
        let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
        if rebase_paths(&mut v, &old_s, &new_s)
            && let Ok(out) = serde_json::to_string_pretty(&v)
        {
            let _ = std::fs::write(f, out);
        }
    }
}

/// Rewrite every string under `v` that starts with the old folder. Returns true if any did.
fn rebase_paths(v: &mut serde_json::Value, old: &str, new: &str) -> bool {
    match v {
        serde_json::Value::String(s) => {
            let hit = s.len() >= old.len()
                && s.is_char_boundary(old.len())
                && s[..old.len()].eq_ignore_ascii_case(old)
                && matches!(s[old.len()..].chars().next(), None | Some('\\' | '/'));
            if hit {
                *s = format!("{new}{}", &s[old.len()..]);
            }
            hit
        }
        serde_json::Value::Array(a) => a.iter_mut().fold(false, |h, x| rebase_paths(x, old, new) | h),
        serde_json::Value::Object(o) => o.values_mut().fold(false, |h, x| rebase_paths(x, old, new) | h),
        _ => false,
    }
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let dst = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &dst)?;
        } else {
            std::fs::copy(e.path(), dst)?;
        }
    }
    Ok(())
}

fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(settings_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if std::env::var_os("MAXCAP_SMOKE").is_some() {
            return; // test runs never touch the user's settings
        }
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let tmp = dir.join("settings.json.tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(&tmp, settings_path());
            }
        }
    }

    /// Room the frame needs outside the video rect (pixels), so it's never clipped.
    pub fn frame_space(&self) -> f32 {
        match self.frame_style {
            FrameStyle::None => 0.0,
            FrameStyle::Outline => self.frame_thickness,
            FrameStyle::PixelCard => self.frame_thickness + 6.0,
            FrameStyle::HardShadow => 8.0,
            FrameStyle::Image => (self.frame_thickness - self.frame_overlap).max(0.0),
        }
    }

    /// Copy a preset's look onto these settings, keeping machine-specific fields
    /// (capture/audio devices, window size, screenshot folder, UI scale).
    pub fn apply_preset(&mut self, preset: &Settings) {
        let keep = self.clone();
        *self = preset.clone();
        self.device_link = keep.device_link;
        self.device_name = keep.device_name;
        self.audio_input = keep.audio_input;
        self.audio_output = keep.audio_output;
        self.window_size = keep.window_size;
        self.screenshot_dir = keep.screenshot_dir;
        self.ui_scale = keep.ui_scale;
        self.language = keep.language;
        self.start_fullscreen = keep.start_fullscreen;
        self.hud_pos = keep.hud_pos;
        self.ghost_opacity = keep.ghost_opacity;
        self.device_profiles = keep.device_profiles;
    }

    /// The parts of these settings a preset stores (machine-specific fields cleared).
    pub fn as_preset(&self) -> Settings {
        let mut p = self.clone();
        p.device_link = None;
        p.device_name = None;
        p.audio_input = None;
        p.audio_output = None;
        p.window_size = None;
        p.screenshot_dir = None;
        p.hud_pos = None;
        p.device_profiles.clear();
        p
    }

    /// True if applying `preset` would change nothing.
    pub fn matches_preset(&self, preset: &Settings) -> bool {
        let mut probe = self.clone();
        probe.apply_preset(preset);
        probe == *self
    }

    pub fn screenshot_dir(&self) -> PathBuf {
        if let Some(d) = &self.screenshot_dir {
            return PathBuf::from(d);
        }
        dirs::picture_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("MaxCap")
    }
}

// ---------------------------------------------------------------------------------
// Presets: %APPDATA%\MaxCap\presets\<name>.json

pub fn presets_dir() -> PathBuf {
    config_dir().join("presets")
}

/// Assets (background/frame images) copied in so presets stay valid if originals move.
pub fn images_dir() -> PathBuf {
    config_dir().join("images")
}

/// Windows-safe file stem for a preset name ("16:10" → "16_10").
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || " -_.()".contains(c) { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_owned()
}

/// A preset on disk. The **file** is the identity: renaming, copying or deleting
/// files in the presets folder is reflected directly.
#[derive(Clone, Debug, PartialEq)]
pub struct PresetEntry {
    /// Display name: the name stored inside the file when it still matches the file
    /// name (keeps characters Windows forbids, like ':'), otherwise the file name.
    pub name: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct PresetFile {
    #[serde(default)]
    name: String,
    settings: Settings,
}

/// Accepts MaxCap preset files, and plain settings JSON (e.g. a copied settings.json).
fn parse_preset(text: &str) -> Option<(String, Settings)> {
    if let Ok(p) = serde_json::from_str::<PresetFile>(text) {
        return Some((p.name, p.settings));
    }
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if !v.is_object() || v.get("settings").is_some() {
        return None;
    }
    serde_json::from_value::<Settings>(v).ok().map(|st| (String::new(), st))
}

/// Presets sorted case-insensitively by display name (Ctrl+1..9 follow this order).
pub fn list_presets() -> Vec<PresetEntry> {
    let mut out: Vec<PresetEntry> = std::fs::read_dir(presets_dir())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("json")))
        .filter_map(|path| {
            let stem = path.file_stem()?.to_string_lossy().into_owned();
            let (inner, _) = parse_preset(&std::fs::read_to_string(&path).ok()?)?;
            let name = if !inner.is_empty() && sanitize(&inner) == stem { inner } else { stem };
            Some(PresetEntry { name, path })
        })
        .collect();
    out.sort_by_key(|e| e.name.to_lowercase());
    out
}

pub fn load_preset(path: &std::path::Path) -> Option<Settings> {
    parse_preset(&std::fs::read_to_string(path).ok()?).map(|(_, s)| s)
}

fn write_preset(path: &std::path::Path, name: &str, settings: &Settings) -> Result<(), String> {
    std::fs::create_dir_all(presets_dir()).map_err(|e| e.to_string())?;
    let file = PresetFile { name: name.to_owned(), settings: settings.as_preset() };
    let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Save under a (new or existing) name; returns the entry written.
pub fn save_preset(name: &str, settings: &Settings) -> Result<PresetEntry, String> {
    let name = name.trim();
    let stem = sanitize(name);
    if stem.is_empty() {
        return Err("Preset needs a name".into());
    }
    let path = presets_dir().join(format!("{stem}.json"));
    write_preset(&path, name, settings)?;
    Ok(PresetEntry { name: name.to_owned(), path })
}

/// Overwrite an existing preset file in place (keeps its file name).
pub fn update_preset(entry: &PresetEntry, settings: &Settings) -> Result<(), String> {
    write_preset(&entry.path, &entry.name, settings)
}

pub fn delete_preset(path: &std::path::Path) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------------
// Import / export: one JSON file holding any number of profiles.

#[derive(Serialize, Deserialize)]
struct ProfileBundle {
    maxcap_profiles: u32,
    profiles: Vec<PresetFile>,
}

/// Serialise profiles (device-specific fields stripped) into one shareable file.
pub fn bundle_to_json(profiles: &[(String, Settings)]) -> Result<String, String> {
    let b = ProfileBundle {
        maxcap_profiles: 1,
        profiles: profiles
            .iter()
            .map(|(n, s)| PresetFile { name: n.clone(), settings: s.as_preset() })
            .collect(),
    };
    serde_json::to_string_pretty(&b).map_err(|e| e.to_string())
}

/// Read a bundle, or a single preset / plain settings.json (named `fallback_name`).
pub fn bundle_from_json(text: &str, fallback_name: &str) -> Result<Vec<(String, Settings)>, String> {
    if let Ok(b) = serde_json::from_str::<ProfileBundle>(text) {
        return Ok(b
            .profiles
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                let n = if p.name.trim().is_empty() { format!("{fallback_name} {}", i + 1) } else { p.name };
                (n, p.settings)
            })
            .collect());
    }
    match parse_preset(text) {
        Some((n, s)) => Ok(vec![(if n.trim().is_empty() { fallback_name.to_owned() } else { n }, s)]),
        None => Err("not a MaxCap profile file".into()),
    }
}

/// Write every saved preset to `path`. Returns how many were exported.
pub fn export_profiles(path: &std::path::Path) -> Result<usize, String> {
    let all: Vec<(String, Settings)> = list_presets()
        .into_iter()
        .filter_map(|e| load_preset(&e.path).map(|s| (e.name, s)))
        .collect();
    if all.is_empty() {
        return Err("no presets to export".into());
    }
    std::fs::write(path, bundle_to_json(&all)?).map_err(|e| e.to_string())?;
    Ok(all.len())
}

/// Add the profiles in `path` as presets; existing names are kept ("Name (2)").
pub fn import_profiles(path: &std::path::Path) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let fallback = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "Imported".into());
    let profiles = bundle_from_json(&text, &fallback)?;
    let mut n = 0;
    for (name, st) in &profiles {
        let mut unique = name.trim().to_owned();
        let mut k = 2;
        while presets_dir().join(format!("{}.json", sanitize(&unique))).exists() {
            unique = format!("{} ({k})", name.trim());
            k += 1;
        }
        save_preset(&unique, st)?;
        n += 1;
    }
    Ok(n)
}

/// Ready-made profiles built from the existing options. Written once as normal preset
/// files (so Ctrl+1..9, rename and delete all work); never recreated once seeded.
fn builtin_profiles() -> Vec<(&'static str, Settings)> {
    let d = Settings::default;
    let mut v = Vec::new();

    let mut s = d();
    s.scale_filter = ScaleFilter::Lanczos2;
    s.present_mode = PresentMode::Immediate;
    s.frame_latency = 1;
    s.sharpen = 0.0;
    s.audio_nr = false;
    s.audio_buffer_ms = 20.0;
    s.hud = HudItems::PRESETS[1].1;
    s.show_stats = true;
    v.push(("Gaming - low latency", s));

    let mut s = d();
    s.scale_filter = ScaleFilter::Lanczos3;
    s.antiring = 0.8;
    s.sharpen = 0.2;
    s.present_mode = PresentMode::Mailbox;
    s.output_depth = OutputDepth::Bits10;
    s.dither = true;
    s.background_mode = BackgroundMode::Ambient;
    s.ambient_style = AmbientStyle::Glow;
    s.audio_buffer_ms = 60.0;
    v.push(("Cinema - quality", s));

    let mut s = d();
    s.scale_filter = ScaleFilter::Nearest;
    s.aspect_mode = AspectMode::Integer;
    s.sharpen = 0.0;
    s.frame_style = FrameStyle::PixelCard;
    s.present_mode = PresentMode::Immediate;
    v.push(("Retro pixel-perfect", s));

    let mut s = d();
    s.scale_filter = ScaleFilter::SharpBilinear;
    s.aspect_mode = AspectMode::Fit;
    s.aspect_override = AspectOverride::R4_3;
    s.saturation = 1.1;
    s.background_mode = BackgroundMode::Ambient;
    s.ambient_style = AmbientStyle::Mirror;
    v.push(("CRT era 4:3", s));

    let mut s = d();
    s.scale_filter = ScaleFilter::Lanczos3;
    s.upscaler = Upscaler::RtxVsr;
    s.upscale_target = UpscaleTarget::Window;
    s.sharpen = 0.15;
    v.push(("AI upscale (RTX)", s));

    let mut s = d();
    s.scale_filter = ScaleFilter::Lanczos2;
    s.frame_gen = true;
    s.frame_gen_factor = 2;
    s.present_mode = PresentMode::Fifo;
    s.audio_buffer_ms = 80.0;
    v.push(("Smooth motion (frame gen)", s));

    let mut s = d();
    s.always_on_top = true;
    s.scale_filter = ScaleFilter::Bilinear;
    s.audio_enabled = true;
    s.audio_buffer_ms = 20.0;
    s.hud = HudItems::PRESETS[1].1;
    s.show_stats = true;
    v.push(("Streaming / OBS window", s));
    v
}

/// Create each built-in profile once. `.profiles-seeded` lists the names already
/// created, so a profile you delete stays deleted and new built-ins still arrive.
pub fn seed_builtin_presets() {
    if std::env::var_os("MAXCAP_SMOKE").is_some() {
        return;
    }
    let marker = config_dir().join(".profiles-seeded");
    let mut seeded: Vec<String> = std::fs::read_to_string(&marker)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
    let mut changed = false;
    for (name, st) in builtin_profiles() {
        if seeded.iter().any(|n| n == name) {
            continue;
        }
        if !presets_dir().join(format!("{}.json", sanitize(name))).exists() {
            let _ = save_preset(name, &st);
        }
        seeded.push(name.to_owned());
        changed = true;
    }
    if changed {
        let _ = std::fs::create_dir_all(config_dir());
        let _ = std::fs::write(marker, seeded.join("\n"));
    }
}

/// Copy a picked image into the app's images folder; returns the stored path.
pub fn import_image(src: &std::path::Path) -> Result<String, String> {
    let dir = images_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = src.file_name().ok_or("bad file name")?;
    let dst = dir.join(name);
    if dst != src {
        std::fs::copy(src, &dst).map_err(|e| e.to_string())?;
    }
    Ok(dst.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_roundtrip_rename_and_plain_settings() {
        let dir = std::env::temp_dir().join(format!("maxcap_preset_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // SAFETY: only this test touches MAXCAP_CONFIG_DIR.
        unsafe { std::env::set_var("MAXCAP_CONFIG_DIR", &dir) };

        let mut mine = Settings::default();
        mine.device_name = Some("USB3 Video".into());
        mine.audio_output = Some("MOTU".into());
        mine.window_size = Some([1600, 900]);

        let mut look = Settings::default();
        look.scale_filter = ScaleFilter::Nearest;
        look.aspect_mode = AspectMode::Integer;
        look.frame_style = FrameStyle::PixelCard;
        look.saturation = 1.2;
        look.device_name = Some("Other Card".into());
        let e = save_preset("Retro 16:10", &look).unwrap();
        assert_eq!(e.path.file_name().unwrap(), "Retro 16_10.json");

        // Display name keeps the ':' while the file still matches.
        let list = list_presets();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Retro 16:10");
        let loaded = load_preset(&list[0].path).unwrap();
        assert_eq!(loaded.device_name, None, "presets never store devices");

        mine.apply_preset(&loaded);
        assert_eq!(mine.scale_filter, ScaleFilter::Nearest);
        assert_eq!(mine.frame_style, FrameStyle::PixelCard);
        assert_eq!(mine.device_name.as_deref(), Some("USB3 Video"), "device kept");
        assert_eq!(mine.audio_output.as_deref(), Some("MOTU"), "audio device kept");
        assert_eq!(mine.window_size, Some([1600, 900]), "window size kept");
        assert!(mine.matches_preset(&loaded));
        mine.saturation = 1.0;
        assert!(!mine.matches_preset(&loaded), "edits mark the preset modified");

        // Renamed in Explorer: listed and loadable under the new file name.
        let renamed = presets_dir().join("Arcade.json");
        std::fs::rename(&list[0].path, &renamed).unwrap();
        let list = list_presets();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Arcade");
        assert_eq!(list[0].path, renamed);
        assert!(load_preset(&list[0].path).is_some());

        // Update keeps the renamed file and its name.
        update_preset(&list[0], &mine).unwrap();
        assert_eq!(list_presets()[0].name, "Arcade");

        // A plain settings.json dropped into the folder works too.
        std::fs::write(presets_dir().join("Copied.json"), serde_json::to_string(&look).unwrap()).unwrap();
        let names: Vec<String> = list_presets().into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["Arcade".to_string(), "Copied".to_string()]);

        for e in list_presets() {
            delete_preset(&e.path).unwrap();
        }
        assert!(list_presets().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundle_roundtrip_and_single_file() {
        let mut a = Settings::default();
        a.scale_filter = ScaleFilter::Nearest;
        a.device_name = Some("Card".into());
        a.hud_pos = Some([0.5, 0.5]);
        a.device_profiles.insert("Card".into(), "X".into());
        let b = Settings::default();
        let json = bundle_to_json(&[("A".into(), a.clone()), ("B".into(), b)]).unwrap();
        let back = bundle_from_json(&json, "fb").unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].0, "A");
        assert_eq!(back[0].1.scale_filter, ScaleFilter::Nearest);
        assert_eq!(back[0].1.device_name, None, "devices are stripped on export");
        assert_eq!(back[0].1.hud_pos, None);
        assert!(back[0].1.device_profiles.is_empty());
        // A plain settings.json imports as one profile named after the file.
        let plain = serde_json::to_string(&a).unwrap();
        let one = bundle_from_json(&plain, "mine").unwrap();
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].0, "mine");
        assert!(bundle_from_json("garbage", "x").is_err());
    }

    #[test]
    fn preset_keeps_card_links_and_hud_position() {
        let mut mine = Settings::default();
        mine.hud_pos = Some([0.2, 0.3]);
        mine.device_profiles.insert("Card".into(), "Gaming".into());
        let mut look = Settings::default();
        look.saturation = 1.3;
        mine.apply_preset(&look);
        assert_eq!(mine.hud_pos, Some([0.2, 0.3]));
        assert_eq!(mine.device_profiles.get("Card").map(String::as_str), Some("Gaming"));
        assert!(mine.matches_preset(&look));
    }

    #[test]
    fn legacy_dir_migrates_and_rebases_image_paths() {
        let root = std::env::temp_dir().join(format!("maxcap_migrate_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (old, new) = (root.join("capview"), root.join("MaxCap"));
        std::fs::create_dir_all(old.join("presets")).unwrap();
        std::fs::create_dir_all(old.join("images")).unwrap();
        std::fs::write(old.join("images").join("bg.jpg"), b"x").unwrap();
        let img = old.join("images").join("bg.jpg").to_string_lossy().into_owned();
        let elsewhere = root.join("capview-other").join("a.png").to_string_lossy().into_owned();
        let preset = serde_json::json!({ "background_image": img, "frame_image": elsewhere });
        std::fs::write(old.join("presets").join("P.json"), preset.to_string()).unwrap();
        std::fs::write(old.join("settings.json"), r#"{"background_image": null}"#).unwrap();

        migrate_legacy_dir(&old, &new);

        assert!(!old.exists());
        assert!(new.join("images").join("bg.jpg").exists());
        let p: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(new.join("presets").join("P.json")).unwrap()).unwrap();
        let want = new.join("images").join("bg.jpg").to_string_lossy().into_owned();
        assert_eq!(p["background_image"], serde_json::json!(want));
        // A sibling folder that merely shares the prefix is left alone.
        assert_eq!(p["frame_image"], serde_json::json!(elsewhere));

        // Never overwrites an existing new folder.
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("settings.json"), "{}").unwrap();
        migrate_legacy_dir(&old, &new);
        assert!(old.join("settings.json").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
