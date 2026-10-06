//! The whole editor (plan.md C4). Version 2: background, top strip, plain pedal bodies.

use crate::editor::paint::Canvas;
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
    for p in PEDALS {
        cv.pedal_shadow(p.x, p.y, p.w, PEDAL_H, 6.0);
        cv.fill_rect(p.x, p.y, p.w, PEDAL_H, 6.0, p.finish.body);
    }
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
