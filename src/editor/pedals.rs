//! Controls of the six on/off pedals (plan.md C4.2, C4.3).

use crate::editor::knob::{knob, KnobSpec, KNOB_SLOT_W};
use crate::editor::layout::{right_run, space_around};
use crate::editor::paint::{Canvas, Style};
use crate::editor::pedal::{frame, on_footswitch, rows, LedState};
use crate::editor::theme::*;
use crate::editor::widgets::{selector, selector_width, toggle, toggle_width};
use crate::editor::UiState;
use crate::params::{DriveVoice, EchoMode, HolyMolyParams, RingMode, VibeMode};
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

/// Centre x of `count` knob slots spread with space-around.
fn knob_slots(g: &PedalGeom, count: usize) -> Vec<f32> {
    let items = vec![KNOB_SLOT_W; count];
    space_around(g.x + 12.0, g.w - 24.0, &items, 8.0)
        .into_iter()
        .map(|x| x + KNOB_SLOT_W / 2.0)
        .collect()
}

fn sel_width(cv: &Canvas, labels: &[&str], label: &str) -> f32 {
    selector_width(cv, labels).max(cv.text_width(label, Style::Label))
}

fn led_on(on: bool) -> LedState {
    LedState {
        on,
        decay: false,
        blink: false,
    }
}

pub fn drive(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
) {
    let g = &DRIVE;
    let d = &p.drive;
    let print = g.finish.print;
    let on = d.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 38.0]);
    let xs = knob_slots(g, 3);
    knob(
        ui,
        cv,
        setter,
        st,
        &d.dist,
        xs[0],
        ys[0],
        KnobSpec::md("drv_dist", "Dist", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &d.tone,
        xs[1],
        ys[0],
        KnobSpec::md("drv_tone", "Tone", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &d.level,
        xs[2],
        ys[0],
        KnobSpec::md("drv_level", "Level", print),
    );
    let labels = ["Tight", "Classic", "Loose"];
    let sw = sel_width(cv, &labels, "Voice");
    let sx = space_around(g.x + 12.0, g.w - 24.0, &[sw], 8.0)[0];
    let opts = [
        (DriveVoice::Tight, labels[0]),
        (DriveVoice::Classic, labels[1]),
        (DriveVoice::Loose, labels[2]),
    ];
    selector(
        ui,
        cv,
        setter,
        &d.voice,
        "drv_voice",
        &opts,
        sx,
        ys[1],
        Some("Voice"),
        print,
        false,
    );
    on_footswitch(ui, cv, setter, st, g, &d.on, "drv_on");
}

pub fn fuzz(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
) {
    let g = &FUZZ;
    let f = &p.fuzz;
    let print = g.finish.print;
    let on = f.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let xs = knob_slots(g, 2);
    knob(
        ui,
        cv,
        setter,
        st,
        &f.fuzz,
        xs[0],
        ys[0],
        KnobSpec::md("fuz_fuzz", "Fuzz", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &f.bias,
        xs[1],
        ys[0],
        KnobSpec::md("fuz_bias", "Bias", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &f.tone,
        xs[0],
        ys[1],
        KnobSpec::md("fuz_tone", "Tone", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &f.volume,
        xs[1],
        ys[1],
        KnobSpec::md("fuz_volume", "Volume", print),
    );
    on_footswitch(ui, cv, setter, st, g, &f.on, "fuz_on");
}

pub fn ring(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
) {
    let g = &RING;
    let r = &p.ring;
    let print = g.finish.print;
    let on = r.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let xs = knob_slots(g, 4);
    knob(
        ui,
        cv,
        setter,
        st,
        &r.gain,
        xs[0],
        ys[0],
        KnobSpec::md("rng_gain", "Gain", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &r.dist,
        xs[1],
        ys[0],
        KnobSpec::md("rng_dist", "Dist", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &r.modulation,
        xs[2],
        ys[0],
        KnobSpec::md("rng_mod", "Mod", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &r.output,
        xs[3],
        ys[0],
        KnobSpec::md("rng_output", "Output", print),
    );
    let labels = ["Self", "Osc"];
    let sw = sel_width(cv, &labels, "Mode");
    let pos = space_around(g.x + 12.0, g.w - 24.0, &[sw, KNOB_SLOT_W], 8.0);
    let opts = [(RingMode::SelfMod, labels[0]), (RingMode::Osc, labels[1])];
    selector(
        ui,
        cv,
        setter,
        &r.mode,
        "rng_mode",
        &opts,
        pos[0],
        ys[1] + 84.0 - 38.0,
        Some("Mode"),
        print,
        false,
    );
    let self_mode = r.mode.value() == RingMode::SelfMod;
    knob(
        ui,
        cv,
        setter,
        st,
        &r.freq,
        pos[1] + KNOB_SLOT_W / 2.0,
        ys[1],
        KnobSpec {
            disabled: self_mode,
            ..KnobSpec::md("rng_freq", "Freq", print)
        },
    );
    on_footswitch(ui, cv, setter, st, g, &r.on, "rng_on");
}

pub fn vibe(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
) {
    let g = &VIBE;
    let v = &p.vibe;
    let print = g.finish.print;
    let on = v.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let xs = knob_slots(g, 3);
    knob(
        ui,
        cv,
        setter,
        st,
        &v.rate,
        xs[0],
        ys[0],
        KnobSpec::md("vib_rate", "Rate", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &v.depth,
        xs[1],
        ys[0],
        KnobSpec::md("vib_depth", "Depth", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &v.feedback,
        xs[2],
        ys[0],
        KnobSpec::md("vib_feedback", "Feedback", print),
    );
    let labels = ["Chorus", "Vibrato"];
    let sw = sel_width(cv, &labels, "Mode");
    let pos = space_around(g.x + 12.0, g.w - 24.0, &[KNOB_SLOT_W, sw], 8.0);
    let vibrato = v.mode.value() == VibeMode::Vibrato;
    knob(
        ui,
        cv,
        setter,
        st,
        &v.mix,
        pos[0] + KNOB_SLOT_W / 2.0,
        ys[1],
        KnobSpec {
            disabled: vibrato,
            ..KnobSpec::md("vib_mix", "Mix", print)
        },
    );
    let opts = [
        (VibeMode::Chorus, labels[0]),
        (VibeMode::Vibrato, labels[1]),
    ];
    selector(
        ui,
        cv,
        setter,
        &v.mode,
        "vib_mode",
        &opts,
        pos[1],
        ys[1] + 84.0 - 38.0,
        Some("Mode"),
        print,
        false,
    );
    on_footswitch(ui, cv, setter, st, g, &v.on, "vib_on");
}

pub fn prism(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
) {
    let g = &PRISM;
    let z = &p.prism;
    let print = g.finish.print;
    let on = z.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let a = knob_slots(g, 4);
    knob(
        ui,
        cv,
        setter,
        st,
        &z.pitch,
        a[0],
        ys[0],
        KnobSpec {
            bipolar: true,
            ..KnobSpec::md("prz_pitch", "Pitch", print)
        },
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &z.primary,
        a[1],
        ys[0],
        KnobSpec::md("prz_primary", "Primary", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &z.secondary,
        a[2],
        ys[0],
        KnobSpec::md("prz_secondary", "Secondary", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &z.halo,
        a[3],
        ys[0],
        KnobSpec::md("prz_halo", "Halo", print),
    );
    let b = knob_slots(g, 3);
    knob(
        ui,
        cv,
        setter,
        st,
        &z.tracking,
        b[0],
        ys[1],
        KnobSpec::md("prz_tracking", "Tracking", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &z.depth,
        b[1],
        ys[1],
        KnobSpec::md("prz_depth", "Depth", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &z.tone,
        b[2],
        ys[1],
        KnobSpec::md("prz_tone", "Tone", print),
    );
    on_footswitch(ui, cv, setter, st, g, &z.on, "prz_on");
}

pub fn echo(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
) {
    let g = &ECHO;
    let e = &p.echo;
    let print = g.finish.print;
    let on = e.on.value();
    let sync = e.sync.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 38.0]);
    let xs = knob_slots(g, 6);
    knob(
        ui,
        cv,
        setter,
        st,
        &e.time,
        xs[0],
        ys[0],
        KnobSpec {
            disabled: sync,
            ..KnobSpec::md("ech_time", "Time", print)
        },
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &e.division,
        xs[1],
        ys[0],
        KnobSpec {
            steps: Some(13),
            disabled: !sync,
            ..KnobSpec::md("ech_division", "Division", print)
        },
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &e.feedback,
        xs[2],
        ys[0],
        KnobSpec {
            danger: Some(1.0 / 1.1),
            ..KnobSpec::md("ech_feedback", "Feedback", print)
        },
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &e.mix,
        xs[3],
        ys[0],
        KnobSpec::md("ech_mix", "Mix", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &e.tone,
        xs[4],
        ys[0],
        KnobSpec::md("ech_tone", "Tone", print),
    );
    knob(
        ui,
        cv,
        setter,
        st,
        &e.modulation,
        xs[5],
        ys[0],
        KnobSpec::md("ech_mod", "Mod", print),
    );
    let opts = [
        (EchoMode::Digital, "Digital"),
        (EchoMode::Tape, "Tape"),
        (EchoMode::Multi, "Multi"),
        (EchoMode::Reverse, "Reverse"),
    ];
    selector(
        ui,
        cv,
        setter,
        &e.mode,
        "ech_mode",
        &opts,
        g.x + 12.0,
        ys[1],
        Some("Mode"),
        print,
        false,
    );
    let labels = ["SYNC", "PING-PONG", "TRAILS"];
    let widths: Vec<f32> = labels.iter().map(|l| toggle_width(cv, l)).collect();
    let tx = right_run(g.x + g.w - 12.0, &widths, 6.0);
    let ty = ys[1] + 8.0;
    toggle(
        ui, cv, setter, &e.sync, "ech_sync", tx[0], ty, widths[0], labels[0], false,
    );
    toggle(
        ui,
        cv,
        setter,
        &e.pingpong,
        "ech_pingpong",
        tx[1],
        ty,
        widths[1],
        labels[1],
        false,
    );
    toggle(
        ui,
        cv,
        setter,
        &e.trails,
        "ech_trails",
        tx[2],
        ty,
        widths[2],
        labels[2],
        false,
    );
    on_footswitch(ui, cv, setter, st, g, &e.on, "ech_on");
}
