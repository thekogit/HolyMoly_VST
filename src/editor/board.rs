//! The whole editor (plan.md C4). Version 1: background, strip body, pedal bodies.

use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{self, pos2, vec2, Rect};

pub fn draw(
    ui: &mut egui::Ui,
    _setter: &ParamSetter,
    _st: &mut UiState,
    _params: &HolyMolyParams,
    _shared: &Shared,
) {
    let painter = ui.painter().clone();
    let o = ui.max_rect().min.to_vec2();
    let rect = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(pos2(x, y) + o, vec2(w, h));
    painter.rect_filled(rect(0.0, 0.0, EDITOR_W, EDITOR_H), 0.0, GROUND);
    for i in 0..94 {
        for j in 0..64 {
            painter.rect_filled(
                rect(5.0 + 12.0 * i as f32, 5.0 + 12.0 * j as f32, 2.0, 2.0),
                0.0,
                LINE,
            );
        }
    }
    painter.rect_filled(rect(16.0, 12.0, 1088.0, 112.0), 6.0, DECK);
    for p in PEDALS {
        painter.rect_filled(rect(p.x, p.y, p.w, PEDAL_H), 6.0, p.finish.body);
    }
}
