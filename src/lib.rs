// SCAFFOLD: process() is a pass-through until Step S16.
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

pub mod dsp;
pub mod fmt;
pub mod params;

use nih_plug::prelude::*;
use params::HolyMolyParams;
use std::sync::Arc;

pub struct HolyMoly {
    params: Arc<HolyMolyParams>,
}

impl Default for HolyMoly {
    fn default() -> Self {
        Self {
            params: Arc::new(HolyMolyParams::default()),
        }
    }
}

impl Plugin for HolyMoly {
    const NAME: &'static str = "Holy Moly";
    const VENDOR: &'static str = "Holy Moly Audio";
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

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        ProcessStatus::Normal
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
