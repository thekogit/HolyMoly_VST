//! Editor (plan.md B2): everything is painted by hand with egui's Painter.

pub mod board;
pub mod fonts;
pub mod knob;
pub mod layout;
pub mod paint;
pub mod pedal;
pub mod strip;
pub mod theme;
pub mod widgets;

use crate::dsp::tuner::{Reading, Yin, BUFFER};
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use std::sync::Arc;
use std::time::Duration;

/// Editor-side state that lives as long as the window.
pub struct UiState {
    /// Param ID of the knob being dragged.
    pub drag_id: Option<&'static str>,
    /// Unsnapped normalized value during a drag.
    pub drag_norm: f32,
    /// Param ID of the momentary button held down.
    pub held_id: Option<&'static str>,
    pub yin: Yin,
    pub tuner_buf: Vec<f32>,
    pub reading: Option<Reading>,
    pub tuner_written: usize,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            drag_id: None,
            drag_norm: 0.0,
            held_id: None,
            yin: Yin::new(),
            tuner_buf: vec![0.0; BUFFER],
            reading: None,
            tuner_written: 0,
        }
    }
}

pub fn create(params: Arc<HolyMolyParams>, shared: Arc<Shared>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        UiState::new(),
        |ctx, _state| fonts::install(ctx),
        move |ctx, setter, state| {
            egui::CentralPanel::default()
                .frame(egui::Frame::default())
                .show(ctx, |ui| {
                    board::draw(ui, setter, state, &params, &shared);
                });
            ctx.request_repaint_after(Duration::from_millis(33));
        },
    )
}
