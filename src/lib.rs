#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]

pub mod chain;
pub mod dsp;
pub mod editor;
pub mod fmt;
pub mod params;
pub mod shared;

use chain::{Chain, ChainSettings, LoopButton};
use dsp::drive::DriveSettings;
use dsp::echo::EchoSettings;
use dsp::fuzz::FuzzSettings;
use dsp::looper::LooperSettings;
use dsp::prism::PrismSettings;
use dsp::ring::RingSettings;
use dsp::vibe::VibeSettings;
use nih_plug::prelude::*;
use params::HolyMolyParams;
use shared::Shared;
use std::sync::Arc;

pub struct HolyMoly {
    params: Arc<HolyMolyParams>,
    shared: Arc<Shared>,
    chain: Chain,
    mono_in: bool,
}

impl Default for HolyMoly {
    fn default() -> Self {
        Self {
            params: Arc::new(HolyMolyParams::default()),
            shared: Arc::new(Shared::new()),
            chain: Chain::new(),
            mono_in: false,
        }
    }
}

impl HolyMoly {
    /// Reads every parameter once per host block (plan.md B3).
    fn settings(&self, tempo: Option<f64>) -> ChainSettings {
        let p = &self.params;
        ChainSettings {
            input_db: p.input.value(),
            output_db: p.output.value(),
            quality: p.quality.value().factor(),
            tuner_mute: p.tuner.mute.value(),
            drive: DriveSettings {
                on: p.drive.on.value(),
                dist: p.drive.dist.value(),
                tone: p.drive.tone.value(),
                level_db: p.drive.level.value(),
                voice: p.drive.voice.value(),
            },
            fuzz: FuzzSettings {
                on: p.fuzz.on.value(),
                fuzz: p.fuzz.fuzz.value(),
                bias: p.fuzz.bias.value(),
                tone: p.fuzz.tone.value(),
                volume_db: p.fuzz.volume.value(),
            },
            ring: RingSettings {
                on: p.ring.on.value(),
                gain_db: p.ring.gain.value(),
                dist: p.ring.dist.value(),
                modulation: p.ring.modulation.value(),
                output_db: p.ring.output.value(),
                mode: p.ring.mode.value(),
                freq_hz: p.ring.freq.value(),
            },
            vibe: VibeSettings {
                on: p.vibe.on.value(),
                rate_hz: p.vibe.rate.value(),
                depth: p.vibe.depth.value(),
                feedback: p.vibe.feedback.value(),
                mix: p.vibe.mix.value(),
                mode: p.vibe.mode.value(),
            },
            prism: PrismSettings {
                on: p.prism.on.value(),
                pitch_st: p.prism.pitch.value(),
                primary: p.prism.primary.value(),
                secondary: p.prism.secondary.value(),
                tracking_ms: p.prism.tracking.value(),
                depth: p.prism.depth.value(),
                halo: p.prism.halo.value(),
                tone_hz: p.prism.tone.value(),
            },
            echo: EchoSettings {
                on: p.echo.on.value(),
                time_ms: p.echo.time.value(),
                sync: p.echo.sync.value(),
                division: p.echo.division.value(),
                feedback: p.echo.feedback.value(),
                mix: p.echo.mix.value(),
                tone_hz: p.echo.tone.value(),
                modulation: p.echo.modulation.value(),
                mode: p.echo.mode.value(),
                pingpong: p.echo.pingpong.value(),
                trails: p.echo.trails.value(),
                tempo_bpm: tempo.unwrap_or(120.0) as f32,
            },
            looper: LooperSettings {
                level_db: p.looper.level.value(),
                reverse: p.looper.reverse.value(),
            },
            loop_btn: p.looper.loop_btn.value(),
            stop_btn: p.looper.stop_btn.value(),
            clear_btn: p.looper.clear_btn.value(),
        }
    }
}

impl Plugin for HolyMoly {
    const NAME: &'static str = "Holy Moly";
    const VENDOR: &'static str = "Holy Moly";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.shared.clone())
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        self.mono_in = audio_io_layout.main_input_channels.map(|c| c.get()) == Some(1);
        let sr = buffer_config.sample_rate;
        self.chain.prepare(sr);
        self.shared.sample_rate.store(sr);
        self.shared.tuner.set_rate(self.chain.tuner_rate());
        context.set_latency_samples(self.chain.latency() as u32);
        true
    }

    fn reset(&mut self) {
        self.chain.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            if let NoteEvent::NoteOn { note, .. } = event {
                match note {
                    60 => self.chain.press(LoopButton::Loop),
                    61 => self.chain.press(LoopButton::Stop),
                    62 => self.chain.press(LoopButton::Clear),
                    _ => {}
                }
            }
        }
        let s = self.settings(context.transport().tempo);
        let channels = buffer.as_slice();
        if channels.len() < 2 {
            return ProcessStatus::Normal;
        }
        let (first, rest) = channels.split_at_mut(1);
        let left = &mut *first[0];
        let right = &mut *rest[0];
        if self.mono_in {
            right.copy_from_slice(left);
        }
        self.chain.process(left, right, &s, &self.shared);
        ProcessStatus::KeepAlive
    }
}

impl Vst3Plugin for HolyMoly {
    const VST3_CLASS_ID: [u8; 16] = *b"HolyMolyBoard001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
        Vst3SubCategory::Delay,
        Vst3SubCategory::Modulation,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_vst3!(HolyMoly);
