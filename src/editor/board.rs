//! The whole editor (plan.md C4). Version 3: background, strip, pedal frames and footswitches.

use crate::editor::paint::Canvas;
use crate::editor::pedal::{self, LedState};
use crate::editor::strip;
use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::{BoolParam, ParamSetter};
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
    let ons: [(&PedalGeom, &BoolParam, &'static str); 6] = [
        (&DRIVE, &params.drive.on, "drv_on"),
        (&FUZZ, &params.fuzz.on, "fuz_on"),
        (&RING, &params.ring.on, "rng_on"),
        (&VIBE, &params.vibe.on, "vib_on"),
        (&PRISM, &params.prism.on, "prz_on"),
        (&ECHO, &params.echo.on, "ech_on"),
    ];
    for (g, param, id) in ons {
        let on = param.value();
        pedal::frame(
            ui,
            &cv,
            g,
            LedState {
                on,
                decay: false,
                blink: false,
            },
            !on,
            0.0,
        );
        pedal::on_footswitch(ui, &cv, setter, st, g, param, id);
    }
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
