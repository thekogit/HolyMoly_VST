//! Top strip (plan.md C4.4) and tuner display (C4.5).

use crate::dsp::tuner::Reading;
use crate::editor::knob::{knob, KnobSize, KnobSpec};
use crate::editor::paint::{Canvas, Style};
use crate::editor::theme::*;
use crate::editor::widgets::{footswitch, icon, led, selector, FootMode, Icon};
use crate::editor::UiState;
use crate::params::{HolyMolyParams, Quality};
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{self, Align2};

/// Runs YIN on the newest tuner samples whenever 1024 new samples have arrived.
pub fn update_tuner(st: &mut UiState, shared: &Shared) {
    let written = shared.tuner.written();
    if written.wrapping_sub(st.tuner_written) >= 1024 {
        shared.tuner.copy_latest(&mut st.tuner_buf);
        let rate = shared.tuner.rate();
        st.reading = st
            .yin
            .detect(&st.tuner_buf, rate)
            .map(|f| Reading::from_freq(f, 440.0));
        st.tuner_written = written;
    }
}

pub fn draw(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    p: &HolyMolyParams,
    shared: &Shared,
) {
    update_tuner(st, shared);

    cv.pedal_shadow(16.0, 12.0, 1088.0, 112.0, 6.0);
    cv.fill_rect(16.0, 12.0, 1088.0, 112.0, 6.0, DECK);
    let bar = cv.clipped(16.0, 120.0, 1088.0, 4.0);
    bar.fill_rect(16.0, 120.0, 1088.0, 4.0, 0.0, GROUND);
    bar.stripes(16.0, 120.0, 1088.0, 4.0, 6.0, 12.0, ACID);
    cv.pedal_bevel(16.0, 12.0, 1088.0, 112.0, 6.0);

    icon(cv, Icon::Halo, 30.0, 49.0, 34.0, ACID.gamma_multiply(0.25));
    icon(cv, Icon::Halo, 32.0, 51.0, 30.0, ACID);
    cv.text(
        72.0,
        41.0,
        Align2::LEFT_TOP,
        "HOLY MOLY",
        Style::Wordmark,
        ACID,
    );
    cv.text(
        72.0,
        80.0,
        Align2::LEFT_TOP,
        "STEREO PEDALBOARD · 8 STAGES",
        Style::Stage,
        INK_MUTED,
    );

    knob(
        ui,
        cv,
        setter,
        st,
        &p.input,
        298.0,
        20.0,
        KnobSpec {
            size: KnobSize::Lg,
            bipolar: true,
            ..KnobSpec::md("in_gain", "Input", INK_MUTED)
        },
    );

    tuner_display(cv, st, p);

    knob(
        ui,
        cv,
        setter,
        st,
        &p.tuner.reference,
        822.0,
        27.0,
        KnobSpec {
            size: KnobSize::Sm,
            ..KnobSpec::md("tun_ref", "Ref", INK_MUTED)
        },
    );

    let muted = p.tuner.mute.value();
    led(ui, cv, 883.0, 38.0, muted, true, false);
    footswitch(
        ui,
        cv,
        setter,
        st,
        &p.tuner.mute,
        "tun_mute",
        883.0,
        64.0,
        true,
        true,
        FootMode::Toggle,
        muted,
    );
    cv.text(
        883.0,
        86.0,
        Align2::CENTER_TOP,
        "Mute",
        Style::Label,
        INK_MUTED,
    );

    let opts = [
        (Quality::X1, Quality::X1.label()),
        (Quality::X2, Quality::X2.label()),
        (Quality::X4, Quality::X4.label()),
    ];
    selector(
        ui,
        cv,
        setter,
        &p.quality,
        "quality",
        &opts,
        916.0,
        47.0,
        Some("Quality"),
        INK_MUTED,
        false,
    );

    knob(
        ui,
        cv,
        setter,
        st,
        &p.output,
        1056.0,
        20.0,
        KnobSpec {
            size: KnobSize::Lg,
            ..KnobSpec::md("out_gain", "Output", INK_MUTED)
        },
    );
}

fn tuner_display(cv: &Canvas, st: &UiState, p: &HolyMolyParams) {
    let (x, y, w, h) = (346.0, 24.0, 432.0, 84.0);
    let muted = p.tuner.mute.value();
    let reference = p.tuner.reference.value();
    let reading = st.reading.map(|r| Reading::from_freq(r.freq, reference));
    let in_tune = matches!(reading, Some(r) if r.cents.abs() <= 5.0);

    cv.well(x, y, w, h, 4.0);
    if muted {
        let bar = cv.clipped(x, y, w, 4.0);
        bar.fill_rect(x, y, w, 4.0, 0.0, WELL);
        bar.stripes(x, y, w, 4.0, 5.0, 10.0, DECAY);
        cv.stroke_rect(x, y, w, h, 4.0, 1.0, DECAY);
    }

    // Note and octave.
    let (note, octave) = match reading {
        Some(r) => (r.name().to_string(), r.octave.to_string()),
        None => ("–".to_string(), String::new()),
    };
    if in_tune {
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            cv.text(
                358.0 + dx,
                42.0 + dy,
                Align2::LEFT_TOP,
                &note,
                Style::Note,
                ACID.gamma_multiply(0.2),
            );
        }
    }
    let nr = cv.text(
        358.0,
        42.0,
        Align2::LEFT_TOP,
        &note,
        Style::Note,
        if in_tune { ACID } else { INK },
    );
    if !octave.is_empty() {
        cv.text(
            nr.max.x - cv.origin.x + 2.0,
            84.0,
            Align2::LEFT_BOTTOM,
            &octave,
            Style::Stage,
            INK_MUTED,
        );
    }

    // 21-segment meter.
    let seg_w = (216.0 - 40.0) / 21.0;
    let at = reading
        .map(|r| (r.cents.clamp(-50.0, 50.0) / 5.0).round() as i32)
        .unwrap_or(0);
    for i in -10i32..=10 {
        let sx = 434.0 + (i + 10) as f32 * (seg_w + 2.0);
        let (color, glow) = match reading {
            Some(_) if in_tune && i.abs() <= 1 => (ACID, true),
            Some(_) if !in_tune && i == at => (DECAY, true),
            Some(_) if !in_tune && ((at > 0 && i > 0 && i < at) || (at < 0 && i < 0 && i > at)) => {
                (DECAY_DEEP, false)
            }
            _ => (LINE, false),
        };
        if glow {
            cv.fill_rect(
                sx - 2.0,
                44.0,
                seg_w + 4.0,
                26.0,
                3.0,
                color.gamma_multiply(0.25),
            );
        }
        cv.fill_rect(sx, 46.0, seg_w, 22.0, 2.0, color);
        if i == 0 {
            cv.stroke_rect(sx, 46.0, seg_w, 22.0, 2.0, 1.0, INK_FAINT);
        }
    }
    cv.text(
        434.0,
        74.0,
        Align2::LEFT_TOP,
        "b 50",
        Style::Stage,
        INK_MUTED,
    );
    cv.text(
        542.0,
        74.0,
        Align2::CENTER_TOP,
        "0",
        Style::Stage,
        INK_MUTED,
    );
    cv.text(
        650.0,
        74.0,
        Align2::RIGHT_TOP,
        "50 #",
        Style::Stage,
        INK_MUTED,
    );

    // Side column.
    let (cents_y, ref_y) = if muted { (41.0, 61.0) } else { (50.0, 70.0) };
    let (text, color) = match reading {
        None => ("NO SIGNAL".to_string(), INK_MUTED),
        Some(_) if in_tune => ("IN TUNE".to_string(), ACID),
        Some(r) => {
            let c = r.cents.round() as i32;
            if c > 0 {
                (format!("+{c} ct #"), DECAY)
            } else {
                (format!("{c} ct b"), DECAY)
            }
        }
    };
    cv.text(
        766.0,
        cents_y,
        Align2::RIGHT_TOP,
        &text,
        Style::Readout,
        color,
    );
    cv.text(
        766.0,
        ref_y,
        Align2::RIGHT_TOP,
        &format!("A4 = {} Hz", reference.round() as i32),
        Style::Stage,
        INK_MUTED,
    );
    if muted {
        let tw = cv.text_width("MUTED", Style::Stage);
        cv.fill_rect(766.0 - tw - 12.0, 84.0, tw + 12.0, 14.0, 2.0, DECAY);
        cv.text(
            760.0,
            91.0,
            Align2::RIGHT_CENTER,
            "MUTED",
            Style::Stage,
            ON_DECAY,
        );
    }
}
