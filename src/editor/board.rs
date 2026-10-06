//! The whole editor (plan.md C4). Version 4: everything except the Looper's controls.

use crate::editor::paint::Canvas;
use crate::editor::pedal::{self, LedState};
use crate::editor::pedals;
use crate::editor::strip;
use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

pub fn draw(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    st: &mut UiState,
    params: &HolyMolyParams,
    shared: &Shared,
) {
    let cv = Canvas::new(ui);
    background(&cv);
    strip::draw(ui, &cv, setter, st, params, shared);
    pedals::drive(ui, &cv, setter, st, params);
    pedals::fuzz(ui, &cv, setter, st, params);
    pedals::ring(ui, &cv, setter, st, params);
    pedals::vibe(ui, &cv, setter, st, params);
    pedals::prism(ui, &cv, setter, st, params);
    pedals::echo(ui, &cv, setter, st, params);
    pedal::frame(
        ui,
        &cv,
        &LOOPER,
        LedState {
            on: false,
            decay: false,
            blink: false,
        },
        false,
        0.0,
    );
}

pub fn background(cv: &Canvas) {
    cv.fill_rect(0.0, 0.0, EDITOR_W, EDITOR_H, 0.0, GROUND);
    for i in 0..94 {
        for j in 0..64 {
            cv.fill_rect(
                5.0 + 12.0 * i as f32,
                5.0 + 12.0 * j as f32,
                2.0,
                2.0,
                0.0,
                LINE,
            );
        }
    }
}
