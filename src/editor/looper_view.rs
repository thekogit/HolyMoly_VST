//! Looper pedal and loop display (reference `Looper` pedal and `LoopDisplay`).

use crate::dsp::looper::LoopState;
use crate::editor::knob::{knob, KnobSpec, KNOB_SLOT_W};
use crate::editor::layout::space_around;
use crate::editor::paint::{Canvas, Style};
use crate::editor::pedal::{frame, rows, LedState};
use crate::editor::theme::*;
use crate::editor::widgets::{
    footswitch, icon, push_button, push_width, toggle, toggle_width, FootMode, Icon,
};
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{self, Align2};

fn secs(v: f32) -> String {
    if v < 10.0 {
        format!("0{v:.1}")
    } else {
        format!("{v:.1}")
    }
}

fn loop_display(
    cv: &Canvas,
    x: f32,
    y: f32,
    w: f32,
    state: LoopState,
    length_s: f32,
    frac: f32,
    reverse: bool,
) {
    cv.well(x, y, w, 41.0, 4.0);
    let (word, which, color) = match state {
        LoopState::Empty => ("EMPTY", Icon::Record, INK_FAINT),
        LoopState::Record => ("RECORD", Icon::Record, DECAY),
        LoopState::Play => ("PLAY", Icon::Play, ACID),
        LoopState::Overdub => ("OVERDUB", Icon::Overdub, DECAY),
        LoopState::Stopped => ("STOPPED", Icon::Stop, INK_MUTED),
    };
    icon(cv, which, x + 8.0, y + 9.0, 12.0, color);
    cv.text(
        x + 25.0,
        y + 15.0,
        Align2::LEFT_CENTER,
        word,
        Style::Readout,
        color,
    );
    let time = match state {
        LoopState::Empty => "MAX 30.0 s".to_string(),
        LoopState::Record => format!("{} / 30.0 s", secs(length_s)),
        _ => format!("{} / {} s", secs(frac * length_s), secs(length_s)),
    };
    let time = if reverse && state != LoopState::Empty {
        format!("REV {time}")
    } else {
        time
    };
    cv.text(
        x + w - 8.0,
        y + 15.0,
        Align2::RIGHT_CENTER,
        &time,
        Style::Stage,
        INK_MUTED,
    );
    let fill = match state {
        LoopState::Record => length_s / 30.0,
        LoopState::Empty => 0.0,
        _ => frac,
    }
    .clamp(0.0, 1.0);
    let bar_y = y + 29.0;
    cv.fill_rect(x + 8.0, bar_y, w - 16.0, 4.0, 2.0, LINE);
    let fill_color = match state {
        LoopState::Record | LoopState::Overdub => DECAY,
        LoopState::Stopped => INK_FAINT,
        _ => ACID,
    };
    if fill > 0.0 {
        cv.fill_rect(x + 8.0, bar_y, (w - 16.0) * fill, 4.0, 2.0, fill_color);
    }
}

pub fn looper(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
    shared: &Shared,
) {
    let g = &LOOPER;
    let lp = &p.looper;
    let print = g.finish.print;
    let (state, len, pos) = shared.looper();
    let sr = shared.sample_rate.load().max(1.0);
    let frac = if len > 0 {
        pos as f32 / len as f32
    } else {
        0.0
    };
    let length_s = if state == LoopState::Record {
        pos as f32 / sr
    } else {
        len as f32 / sr
    };
    let led_state = match state {
        LoopState::Record => LedState {
            on: true,
            decay: true,
            blink: true,
        },
        LoopState::Overdub => LedState {
            on: true,
            decay: true,
            blink: false,
        },
        LoopState::Play => LedState {
            on: true,
            decay: false,
            blink: false,
        },
        _ => LedState {
            on: false,
            decay: false,
            blink: false,
        },
    };
    let orbit = if state == LoopState::Empty { 0.0 } else { frac };
    frame(ui, cv, g, led_state, false, orbit);

    let ys = rows(g, &[41.0, 84.0]);
    loop_display(
        cv,
        g.x + 12.0,
        ys[0],
        g.w - 24.0,
        state,
        length_s,
        frac,
        lp.reverse.value(),
    );

    let stop_w = push_width(cv, "STOP");
    let col_w = toggle_width(cv, "REVERSE").max(stop_w + 6.0 + push_width(cv, "CLEAR"));
    let xs = space_around(g.x + 12.0, g.w - 24.0, &[KNOB_SLOT_W, col_w], 8.0);
    knob(
        ui,
        cv,
        setter,
        st,
        &lp.level,
        xs[0] + KNOB_SLOT_W / 2.0,
        ys[1],
        KnobSpec::md("lop_level", "Loop level", print),
    );
    let col_top = ys[1] + 84.0 - 50.0;
    toggle(
        ui,
        cv,
        setter,
        &lp.reverse,
        "lop_reverse",
        xs[1],
        col_top,
        col_w,
        "REVERSE",
        false,
    );
    push_button(
        ui,
        cv,
        setter,
        st,
        &lp.stop_btn,
        "lop_stop",
        xs[1],
        col_top + 28.0,
        "STOP",
        Icon::Stop,
    );
    push_button(
        ui,
        cv,
        setter,
        st,
        &lp.clear_btn,
        "lop_clear",
        xs[1] + stop_w + 6.0,
        col_top + 28.0,
        "CLEAR",
        Icon::Clear,
    );

    let next = match state {
        LoopState::Empty => "REC",
        LoopState::Record => "PLAY",
        LoopState::Play => "DUB",
        LoopState::Overdub => "PLAY",
        LoopState::Stopped => "PLAY",
    };
    let decay = matches!(state, LoopState::Record | LoopState::Overdub);
    let lit = matches!(
        state,
        LoopState::Record | LoopState::Overdub | LoopState::Play
    );
    let fx = g.x + g.w / 2.0 - (52.0 + 12.0 + 52.0) / 2.0 + 26.0;
    let fy = g.y + 262.0;
    footswitch(
        ui,
        cv,
        setter,
        st,
        &lp.loop_btn,
        "lop_loop",
        fx,
        fy,
        false,
        decay,
        FootMode::Momentary,
        lit,
    );
    let tx = fx + 26.0 + 12.0;
    cv.text(
        tx,
        fy - 1.0,
        Align2::LEFT_BOTTOM,
        "NEXT",
        Style::Stage,
        print.gamma_multiply(0.8),
    );
    cv.text(tx, fy, Align2::LEFT_TOP, next, Style::Readout, print);
}
