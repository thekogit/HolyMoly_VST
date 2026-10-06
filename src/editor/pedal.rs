//! Pedal frame and artwork (plan.md C4.2; reference `Pedal` and `PedalArt`).

use crate::editor::layout::stack_center;
use crate::editor::paint::{polar, Canvas, Style};
use crate::editor::theme::*;
use crate::editor::widgets::{footswitch, led, FootMode};
use crate::editor::UiState;
use nih_plug::prelude::{BoolParam, ParamSetter};
use nih_plug_egui::egui::{self, Align2};

#[derive(Clone, Copy, Debug)]
pub struct LedState {
    pub on: bool,
    pub decay: bool,
    pub blink: bool,
}

/// Body, artwork and head row. `bypassed` fades the art; `art_position` drives the Orbit dot (0..1).
pub fn frame(
    ui: &egui::Ui,
    cv: &Canvas,
    g: &PedalGeom,
    led_state: LedState,
    bypassed: bool,
    art_position: f32,
) {
    let (x, y, w) = (g.x, g.y, g.w);
    cv.pedal_shadow(x, y, w, PEDAL_H, 6.0);
    cv.fill_rect(x, y, w, PEDAL_H, 6.0, g.finish.body);
    let fade = if bypassed { 0.18 } else { 1.0 };
    art(&cv.clipped(x, y, w, PEDAL_H), g, fade, art_position);
    cv.pedal_bevel(x, y, w, PEDAL_H, 6.0);

    let print = g.finish.print;
    let bw = cv.text_width(g.stage, Style::Stage) + 8.0;
    cv.stroke_rect(x + 12.0, y + 19.0, bw, 14.0, 2.0, 1.0, print);
    cv.text(
        x + 12.0 + bw / 2.0,
        y + 26.0,
        Align2::CENTER_CENTER,
        g.stage,
        Style::Stage,
        print,
    );
    cv.text(
        x + 12.0 + bw + 8.0,
        y + 26.0,
        Align2::LEFT_CENTER,
        g.name,
        Style::PedalName,
        print,
    );
    led(
        ui,
        cv,
        x + w - 16.0,
        y + 26.0,
        led_state.on,
        led_state.decay,
        led_state.blink,
    );
}

/// Standard footer: one toggle footswitch for the pedal's On parameter.
pub fn on_footswitch(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    g: &PedalGeom,
    param: &BoolParam,
    id: &'static str,
) {
    let on = param.value();
    footswitch(
        ui,
        cv,
        setter,
        st,
        param,
        id,
        g.x + g.w / 2.0,
        g.y + 262.0,
        false,
        false,
        FootMode::Toggle,
        on,
    );
}

/// Top y of each row inside the body area (y+40, height 184, gap 8).
pub fn rows(g: &PedalGeom, heights: &[f32]) -> Vec<f32> {
    stack_center(g.y + 40.0, 184.0, heights, 8.0)
}

fn art(cv: &Canvas, g: &PedalGeom, fade: f32, position: f32) {
    let (x, w) = (g.x, g.w);
    let bottom = g.y + PEDAL_H;
    let print = g.finish.print;
    let col = |opacity: f32| print.gamma_multiply(opacity * fade);
    let top = bottom - 76.0;
    let (cx, cy) = (x + w / 2.0, top + 38.0);
    let svg = cv.clipped(x, top, w, 76.0);
    match g.art {
        Art::Stripes => cv.stripes(x, bottom - 49.0, w, 22.0, 7.0, 14.0, col(1.0)),
        Art::Halftone => {
            let t0 = bottom - 96.0;
            let n = (w / 10.0).ceil() as usize;
            for j in 0..10 {
                let dy = t0 + 5.0 + 10.0 * j as f32;
                let t = (bottom - dy) / 96.0;
                let m = ((1.0 - t) / 0.9).clamp(0.0, 1.0);
                for i in 0..n {
                    cv.circle(x + 5.0 + 10.0 * i as f32, dy, 1.7, col(0.7 * m));
                }
            }
        }
        Art::Diamonds => {
            let t0 = bottom - 96.0;
            let p = 12.0 * std::f32::consts::SQRT_2;
            let count = ((w + 96.0) / p).ceil() as i32 + 2;
            for band in 0..8 {
                let y0 = t0 + 12.0 * band as f32;
                let y1 = y0 + 12.0;
                let t = (bottom - (y0 + 6.0)) / 96.0;
                let m = ((1.0 - t) / 0.8).clamp(0.0, 1.0);
                if m <= 0.0 {
                    continue;
                }
                let c = col(0.55 * m);
                let clip = cv.clipped(x, y0, w, 12.0);
                for k in -10..count {
                    let s = x + k as f32 * p;
                    clip.line(s + (bottom - y1), y1, s + (bottom - y0), y0, 1.5, c);
                    clip.line(
                        s - bottom + y1 + 96.0,
                        y1,
                        s - bottom + y0 + 96.0,
                        y0,
                        1.5,
                        c,
                    );
                }
            }
        }
        Art::Rings => {
            for i in 0..6 {
                svg.ring(cx, cy, 34.0 + 12.0 * i as f32, 1.5, col(0.4));
            }
        }
        Art::Fan => {
            let mut a = -84.0f32;
            while a <= 84.0 {
                let (x0, y0) = polar(cx, cy, 34.0, a);
                let (x1, y1) = polar(cx, cy, 220.0, a);
                svg.line(x0, y0, x1, y1, 1.0, col(0.5));
                a += 12.0;
            }
        }
        Art::Echoes => {
            for k in 0..9 {
                let bh = 44.0 * 0.78f32.powi(k);
                let off = 38.0 + 16.0 * k as f32;
                svg.fill_rect(cx + off - 2.0, cy - bh / 2.0, 4.0, bh, 1.0, col(0.55));
                svg.fill_rect(cx - off - 2.0, cy - bh / 2.0, 4.0, bh, 1.0, col(0.55));
            }
        }
        Art::Orbit => {
            svg.dashed_arc(cx, cy, 33.0, 0.0, 360.0, 1.5, col(0.4), 2.0, 5.0);
            let (px, py) = polar(cx, cy, 33.0, 360.0 * position);
            svg.circle(px, py, 3.0, col(0.4));
        }
    }
}
