//! The 52 host parameters. IDs are FROZEN (plan.md Table C1). Smoothing is done in the DSP (B3).

use crate::fmt;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

pub const EDITOR_WIDTH: u32 = 1120;
pub const EDITOR_HEIGHT: u32 = 760;

/// Skew factor that puts the geometric mean of [min, max] at the knob centre.
pub fn log_skew(min: f32, max: f32) -> f32 {
    let mid = (min * max).sqrt();
    0.5f32.ln() / ((mid - min) / (max - min)).ln()
}

fn pct(name: &str, default: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max })
        .with_value_to_string(Arc::new(fmt::percent))
        .with_string_to_value(Arc::new(fmt::parse_percent))
}

fn db(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::db))
        .with_string_to_value(Arc::new(fmt::parse_db))
}

fn lin_hz(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::hz))
        .with_string_to_value(Arc::new(fmt::parse_hz))
}

fn log_hz(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min,
            max,
            factor: log_skew(min, max),
        },
    )
    .with_value_to_string(Arc::new(fmt::hz))
    .with_string_to_value(Arc::new(fmt::parse_hz))
}

fn lin_ms(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::ms))
        .with_string_to_value(Arc::new(fmt::parse_ms))
}

fn log_ms(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min,
            max,
            factor: log_skew(min, max),
        },
    )
    .with_value_to_string(Arc::new(fmt::ms))
    .with_string_to_value(Arc::new(fmt::parse_ms))
}

fn semis(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::semitones))
        .with_string_to_value(Arc::new(fmt::parse_semitones))
}

fn loop_level(name: &str) -> FloatParam {
    FloatParam::new(
        name,
        0.0,
        FloatRange::Linear {
            min: -60.0,
            max: 6.0,
        },
    )
    .with_value_to_string(Arc::new(fmt::loop_level))
    .with_string_to_value(Arc::new(fmt::parse_loop_level))
}

// ---------------------------------------------------------------- enums

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quality {
    #[id = "1x"]
    #[name = "1×"]
    X1,
    #[id = "2x"]
    #[name = "2×"]
    X2,
    #[id = "4x"]
    #[name = "4×"]
    X4,
}

impl Quality {
    pub const ALL: [Quality; 3] = [Quality::X1, Quality::X2, Quality::X4];
    pub fn factor(self) -> usize {
        match self {
            Quality::X1 => 1,
            Quality::X2 => 2,
            Quality::X4 => 4,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Quality::X1 => "1×",
            Quality::X2 => "2×",
            Quality::X4 => "4×",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DriveVoice {
    #[id = "tight"]
    #[name = "Tight"]
    Tight,
    #[id = "classic"]
    #[name = "Classic"]
    Classic,
    #[id = "loose"]
    #[name = "Loose"]
    Loose,
}

impl DriveVoice {
    pub const ALL: [DriveVoice; 3] = [DriveVoice::Tight, DriveVoice::Classic, DriveVoice::Loose];
    pub fn label(self) -> &'static str {
        match self {
            DriveVoice::Tight => "Tight",
            DriveVoice::Classic => "Classic",
            DriveVoice::Loose => "Loose",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum RingMode {
    #[id = "self"]
    #[name = "Self"]
    SelfMod,
    #[id = "osc"]
    #[name = "Osc"]
    Osc,
}

impl RingMode {
    pub const ALL: [RingMode; 2] = [RingMode::SelfMod, RingMode::Osc];
    pub fn label(self) -> &'static str {
        match self {
            RingMode::SelfMod => "Self",
            RingMode::Osc => "Osc",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VibeMode {
    #[id = "chorus"]
    #[name = "Chorus"]
    Chorus,
    #[id = "vibrato"]
    #[name = "Vibrato"]
    Vibrato,
}

impl VibeMode {
    pub const ALL: [VibeMode; 2] = [VibeMode::Chorus, VibeMode::Vibrato];
    pub fn label(self) -> &'static str {
        match self {
            VibeMode::Chorus => "Chorus",
            VibeMode::Vibrato => "Vibrato",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum EchoMode {
    #[id = "digital"]
    #[name = "Digital"]
    Digital,
    #[id = "tape"]
    #[name = "Tape"]
    Tape,
    #[id = "multi"]
    #[name = "Multi"]
    Multi,
    #[id = "reverse"]
    #[name = "Reverse"]
    Reverse,
}

impl EchoMode {
    pub const ALL: [EchoMode; 4] = [
        EchoMode::Digital,
        EchoMode::Tape,
        EchoMode::Multi,
        EchoMode::Reverse,
    ];
    pub fn label(self) -> &'static str {
        match self {
            EchoMode::Digital => "Digital",
            EchoMode::Tape => "Tape",
            EchoMode::Multi => "Multi",
            EchoMode::Reverse => "Reverse",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Division {
    #[id = "1_1"]
    #[name = "1/1"]
    Whole,
    #[id = "1_2d"]
    #[name = "1/2D"]
    HalfDotted,
    #[id = "1_2"]
    #[name = "1/2"]
    Half,
    #[id = "1_2t"]
    #[name = "1/2T"]
    HalfTriplet,
    #[id = "1_4d"]
    #[name = "1/4D"]
    QuarterDotted,
    #[id = "1_4"]
    #[name = "1/4"]
    Quarter,
    #[id = "1_4t"]
    #[name = "1/4T"]
    QuarterTriplet,
    #[id = "1_8d"]
    #[name = "1/8D"]
    EighthDotted,
    #[id = "1_8"]
    #[name = "1/8"]
    Eighth,
    #[id = "1_8t"]
    #[name = "1/8T"]
    EighthTriplet,
    #[id = "1_16d"]
    #[name = "1/16D"]
    SixteenthDotted,
    #[id = "1_16"]
    #[name = "1/16"]
    Sixteenth,
    #[id = "1_16t"]
    #[name = "1/16T"]
    SixteenthTriplet,
}

impl Division {
    pub const ALL: [Division; 13] = [
        Division::Whole,
        Division::HalfDotted,
        Division::Half,
        Division::HalfTriplet,
        Division::QuarterDotted,
        Division::Quarter,
        Division::QuarterTriplet,
        Division::EighthDotted,
        Division::Eighth,
        Division::EighthTriplet,
        Division::SixteenthDotted,
        Division::Sixteenth,
        Division::SixteenthTriplet,
    ];

    /// Length in quarter-note beats (plan.md C1a).
    pub fn beats(self) -> f32 {
        match self {
            Division::Whole => 4.0,
            Division::HalfDotted => 3.0,
            Division::Half => 2.0,
            Division::HalfTriplet => 4.0 / 3.0,
            Division::QuarterDotted => 1.5,
            Division::Quarter => 1.0,
            Division::QuarterTriplet => 2.0 / 3.0,
            Division::EighthDotted => 0.75,
            Division::Eighth => 0.5,
            Division::EighthTriplet => 1.0 / 3.0,
            Division::SixteenthDotted => 0.375,
            Division::Sixteenth => 0.25,
            Division::SixteenthTriplet => 1.0 / 6.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Division::Whole => "1/1",
            Division::HalfDotted => "1/2D",
            Division::Half => "1/2",
            Division::HalfTriplet => "1/2T",
            Division::QuarterDotted => "1/4D",
            Division::Quarter => "1/4",
            Division::QuarterTriplet => "1/4T",
            Division::EighthDotted => "1/8D",
            Division::Eighth => "1/8",
            Division::EighthTriplet => "1/8T",
            Division::SixteenthDotted => "1/16D",
            Division::Sixteenth => "1/16",
            Division::SixteenthTriplet => "1/16T",
        }
    }

    /// Position in `ALL` (0..=12).
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|d| *d == self).unwrap_or(5)
    }
}

// ---------------------------------------------------------------- parameter tree

#[derive(Params)]
pub struct HolyMolyParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "in_gain"]
    pub input: FloatParam,
    #[id = "out_gain"]
    pub output: FloatParam,
    #[id = "quality"]
    pub quality: EnumParam<Quality>,

    #[nested(group = "Tuner")]
    pub tuner: TunerParams,
    #[nested(group = "Drive")]
    pub drive: DriveParams,
    #[nested(group = "Fuzz")]
    pub fuzz: FuzzParams,
    #[nested(group = "Ring")]
    pub ring: RingParams,
    #[nested(group = "Vibe")]
    pub vibe: VibeParams,
    #[nested(group = "Prism")]
    pub prism: PrismParams,
    #[nested(group = "Echo")]
    pub echo: EchoParams,
    #[nested(group = "Looper")]
    pub looper: LooperParams,
}

impl Default for HolyMolyParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(EDITOR_WIDTH, EDITOR_HEIGHT),
            input: db("Input", -24.0, 24.0, 0.0),
            output: db("Output", -24.0, 12.0, 0.0),
            quality: EnumParam::new("Quality", Quality::X4),
            tuner: TunerParams::default(),
            drive: DriveParams::default(),
            fuzz: FuzzParams::default(),
            ring: RingParams::default(),
            vibe: VibeParams::default(),
            prism: PrismParams::default(),
            echo: EchoParams::default(),
            looper: LooperParams::default(),
        }
    }
}

#[derive(Params)]
pub struct TunerParams {
    #[id = "tun_mute"]
    pub mute: BoolParam,
    #[id = "tun_ref"]
    pub reference: FloatParam,
}

impl Default for TunerParams {
    fn default() -> Self {
        Self {
            mute: BoolParam::new("Mute", false),
            reference: lin_hz("Reference", 430.0, 450.0, 440.0),
        }
    }
}

#[derive(Params)]
pub struct DriveParams {
    #[id = "drv_on"]
    pub on: BoolParam,
    #[id = "drv_dist"]
    pub dist: FloatParam,
    #[id = "drv_tone"]
    pub tone: FloatParam,
    #[id = "drv_level"]
    pub level: FloatParam,
    #[id = "drv_voice"]
    pub voice: EnumParam<DriveVoice>,
}

impl Default for DriveParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Drive On", true),
            dist: pct("Dist", 0.5, 1.0),
            tone: pct("Tone", 0.5, 1.0),
            level: db("Level", -24.0, 12.0, 0.0),
            voice: EnumParam::new("Voice", DriveVoice::Classic),
        }
    }
}

#[derive(Params)]
pub struct FuzzParams {
    #[id = "fuz_on"]
    pub on: BoolParam,
    #[id = "fuz_fuzz"]
    pub fuzz: FloatParam,
    #[id = "fuz_bias"]
    pub bias: FloatParam,
    #[id = "fuz_tone"]
    pub tone: FloatParam,
    #[id = "fuz_volume"]
    pub volume: FloatParam,
}

impl Default for FuzzParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Fuzz On", true),
            fuzz: pct("Fuzz", 0.6, 1.0),
            bias: pct("Bias", 0.7, 1.0),
            tone: pct("Tone", 0.5, 1.0),
            volume: db("Volume", -24.0, 12.0, 0.0),
        }
    }
}

#[derive(Params)]
pub struct RingParams {
    #[id = "rng_on"]
    pub on: BoolParam,
    #[id = "rng_gain"]
    pub gain: FloatParam,
    #[id = "rng_dist"]
    pub dist: FloatParam,
    #[id = "rng_mod"]
    pub modulation: FloatParam,
    #[id = "rng_output"]
    pub output: FloatParam,
    #[id = "rng_mode"]
    pub mode: EnumParam<RingMode>,
    #[id = "rng_freq"]
    pub freq: FloatParam,
}

impl Default for RingParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Ring On", false),
            gain: db("Gain", 0.0, 30.0, 6.0),
            dist: pct("Dist", 0.4, 1.0),
            modulation: pct("Mod", 1.0, 1.0),
            output: db("Output", -24.0, 12.0, 0.0),
            mode: EnumParam::new("Mode", RingMode::SelfMod),
            freq: log_hz("Freq", 20.0, 2000.0, 440.0),
        }
    }
}

#[derive(Params)]
pub struct VibeParams {
    #[id = "vib_on"]
    pub on: BoolParam,
    #[id = "vib_rate"]
    pub rate: FloatParam,
    #[id = "vib_depth"]
    pub depth: FloatParam,
    #[id = "vib_feedback"]
    pub feedback: FloatParam,
    #[id = "vib_mix"]
    pub mix: FloatParam,
    #[id = "vib_mode"]
    pub mode: EnumParam<VibeMode>,
}

impl Default for VibeParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Vibe On", true),
            rate: log_hz("Rate", 0.1, 10.0, 1.5),
            depth: pct("Depth", 0.6, 1.0),
            feedback: pct("Feedback", 0.3, 0.9),
            mix: pct("Mix", 0.5, 1.0),
            mode: EnumParam::new("Mode", VibeMode::Chorus),
        }
    }
}

#[derive(Params)]
pub struct PrismParams {
    #[id = "prz_on"]
    pub on: BoolParam,
    #[id = "prz_pitch"]
    pub pitch: FloatParam,
    #[id = "prz_primary"]
    pub primary: FloatParam,
    #[id = "prz_secondary"]
    pub secondary: FloatParam,
    #[id = "prz_tracking"]
    pub tracking: FloatParam,
    #[id = "prz_depth"]
    pub depth: FloatParam,
    #[id = "prz_halo"]
    pub halo: FloatParam,
    #[id = "prz_tone"]
    pub tone: FloatParam,
}

impl Default for PrismParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Prism On", false),
            pitch: semis("Pitch", -12.0, 12.0, 12.0),
            primary: pct("Primary", 0.5, 1.0),
            secondary: pct("Secondary", 0.0, 1.0),
            tracking: lin_ms("Tracking", 10.0, 120.0, 40.0),
            depth: pct("Depth", 0.2, 1.0),
            halo: pct("Halo", 0.0, 0.95),
            tone: log_hz("Tone", 1000.0, 16000.0, 8000.0),
        }
    }
}

#[derive(Params)]
pub struct EchoParams {
    #[id = "ech_on"]
    pub on: BoolParam,
    #[id = "ech_time"]
    pub time: FloatParam,
    #[id = "ech_sync"]
    pub sync: BoolParam,
    #[id = "ech_division"]
    pub division: EnumParam<Division>,
    #[id = "ech_feedback"]
    pub feedback: FloatParam,
    #[id = "ech_mix"]
    pub mix: FloatParam,
    #[id = "ech_tone"]
    pub tone: FloatParam,
    #[id = "ech_mod"]
    pub modulation: FloatParam,
    #[id = "ech_mode"]
    pub mode: EnumParam<EchoMode>,
    #[id = "ech_pingpong"]
    pub pingpong: BoolParam,
    #[id = "ech_trails"]
    pub trails: BoolParam,
}

impl Default for EchoParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Echo On", true),
            time: log_ms("Time", 10.0, 3000.0, 400.0),
            sync: BoolParam::new("Sync", false),
            division: EnumParam::new("Division", Division::Quarter),
            feedback: pct("Feedback", 0.35, 1.1),
            mix: pct("Mix", 0.35, 1.0),
            tone: log_hz("Tone", 1000.0, 16000.0, 6000.0),
            modulation: pct("Mod", 0.1, 1.0),
            mode: EnumParam::new("Mode", EchoMode::Digital),
            pingpong: BoolParam::new("Ping-pong", false),
            trails: BoolParam::new("Trails", true),
        }
    }
}

#[derive(Params)]
pub struct LooperParams {
    #[id = "lop_loop"]
    pub loop_btn: BoolParam,
    #[id = "lop_stop"]
    pub stop_btn: BoolParam,
    #[id = "lop_clear"]
    pub clear_btn: BoolParam,
    #[id = "lop_level"]
    pub level: FloatParam,
    #[id = "lop_reverse"]
    pub reverse: BoolParam,
}

impl Default for LooperParams {
    fn default() -> Self {
        Self {
            loop_btn: BoolParam::new("Loop", false),
            stop_btn: BoolParam::new("Stop", false),
            clear_btn: BoolParam::new("Clear", false),
            level: loop_level("Loop level"),
            reverse: BoolParam::new("Reverse", false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_IDS: [&str; 52] = [
        "in_gain",
        "out_gain",
        "quality",
        "tun_mute",
        "tun_ref",
        "drv_on",
        "drv_dist",
        "drv_tone",
        "drv_level",
        "drv_voice",
        "fuz_on",
        "fuz_fuzz",
        "fuz_bias",
        "fuz_tone",
        "fuz_volume",
        "rng_on",
        "rng_gain",
        "rng_dist",
        "rng_mod",
        "rng_output",
        "rng_mode",
        "rng_freq",
        "vib_on",
        "vib_rate",
        "vib_depth",
        "vib_feedback",
        "vib_mix",
        "vib_mode",
        "prz_on",
        "prz_pitch",
        "prz_primary",
        "prz_secondary",
        "prz_tracking",
        "prz_depth",
        "prz_halo",
        "prz_tone",
        "ech_on",
        "ech_time",
        "ech_sync",
        "ech_division",
        "ech_feedback",
        "ech_mix",
        "ech_tone",
        "ech_mod",
        "ech_mode",
        "ech_pingpong",
        "ech_trails",
        "lop_loop",
        "lop_stop",
        "lop_clear",
        "lop_level",
        "lop_reverse",
    ];

    #[test]
    fn has_52_params_with_frozen_ids() {
        let p = HolyMolyParams::default();
        let mut ids: Vec<String> = p.param_map().into_iter().map(|(id, _, _)| id).collect();
        ids.sort();
        let mut expected: Vec<String> = EXPECTED_IDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[test]
    fn defaults_match_table_c1() {
        let p = HolyMolyParams::default();
        assert_eq!(p.quality.value(), Quality::X4);
        assert!(!p.ring.on.value());
        assert!(!p.prism.on.value());
        assert!(p.drive.on.value());
        assert!(p.echo.trails.value());
        assert_eq!(p.echo.division.value(), Division::Quarter);
        assert_eq!(p.drive.voice.value(), DriveVoice::Classic);
        assert_eq!(p.ring.mode.value(), RingMode::SelfMod);
        assert!((p.echo.time.value() - 400.0).abs() < 1e-3);
        assert!((p.echo.feedback.value() - 0.35).abs() < 1e-6);
        assert!((p.prism.pitch.value() - 12.0).abs() < 1e-6);
        assert!((p.tuner.reference.value() - 440.0).abs() < 1e-6);
        assert!(p.looper.level.value().abs() < 1e-6);
    }

    #[test]
    fn log_skew_puts_geometric_mean_at_centre() {
        let (min, max) = (20.0f32, 2000.0f32);
        let f = log_skew(min, max);
        let norm = ((200.0 - min) / (max - min)).powf(f);
        assert!((norm - 0.5).abs() < 1e-4);
        assert_eq!(Division::Quarter.index(), 5);
        assert!((Division::EighthDotted.beats() - 0.75).abs() < 1e-6);
    }
}
