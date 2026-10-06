//! Small widgets: LED, footswitch, selector, toggle, push button and icons (reference components).

use crate::editor::paint::{dim, polar, Canvas, Style};
use crate::editor::theme::*;
use crate::editor::UiState;
use nih_plug::prelude::{BoolParam, Enum, EnumParam, ParamSetter};
use nih_plug_egui::egui::{self, Align2, Color32, Id, Mesh, Sense, Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Halo,
    Record,
    Play,
    Overdub,
    Stop,
    Clear,
    Reverse,
}

/// Filled ring sector between radii r0 and r1; angles in degrees clockwise from 12 o'clock.
pub fn sector(cv: &Canvas, cx: f32, cy: f32, r0: f32, r1: f32, a0: f32, a1: f32, color: Color32) {
    let n = (((a1 - a0).abs() / 6.0).ceil() as usize).max(1);
    let mut mesh = Mesh::default();
    for k in 0..=n {
        let a = a0 + (a1 - a0) * k as f32 / n as f32;
        let (xi, yi) = polar(cx, cy, r0, a);
        let (xo, yo) = polar(cx, cy, r1, a);
        mesh.colored_vertex(cv.p(xi, yi), color);
        mesh.colored_vertex(cv.p(xo, yo), color);
    }
    for k in 0..n as u32 {
        let i = 2 * k;
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i + 1, i + 3, i + 2);
    }
    cv.painter.add(Shape::mesh(mesh));
}

/// Draws an icon in a `size` px box whose top-left corner is (x, y). Shapes follow the 24-unit SVG paths.
pub fn icon(cv: &Canvas, which: Icon, x: f32, y: f32, size: f32, color: Color32) {
    let s = size / 24.0;
    let p = |u: f32, v: f32| (x + u * s, y + v * s);
    let (cx, cy) = p(12.0, 12.0);
    match which {
        Icon::Halo => {
            cv.ring(cx, cy, 10.6 * s, 1.2 * s, color);
            for a in [30.0, 150.0, 270.0] {
                sector(cv, cx, cy, 3.6 * s, 9.0 * s, a, a + 60.0, color);
            }
            cv.circle(cx, cy, 2.2 * s, color);
        }
        Icon::Record => cv.circle(cx, cy, 6.0 * s, color),
        Icon::Play => cv.polygon(&[p(8.0, 5.5), p(19.0, 12.0), p(8.0, 18.5)], color),
        Icon::Overdub => {
            cv.ring(cx, cy, 7.0 * s, 2.0 * s, color);
            cv.circle(cx, cy, 3.0 * s, color);
        }
        Icon::Stop => cv.polygon(
            &[p(6.5, 6.5), p(17.5, 6.5), p(17.5, 17.5), p(6.5, 17.5)],
            color,
        ),
        Icon::Clear => {
            let (a, b) = (p(6.0, 6.0), p(18.0, 18.0));
            cv.line(a.0, a.1, b.0, b.1, 2.0 * s, color);
            let (c, d) = (p(18.0, 6.0), p(6.0, 18.0));
            cv.line(c.0, c.1, d.0, d.1, 2.0 * s, color);
        }
        Icon::Reverse => {
            let (a, b) = (p(16.0, 5.5), p(5.0, 12.0));
            cv.polygon(&[a, b, p(16.0, 18.5)], color);
            cv.polygon(
                &[p(17.0, 5.5), p(19.0, 5.5), p(19.0, 18.5), p(17.0, 18.5)],
                color,
            );
        }
    }
}

/// LED: 8 px dot inside a 2 px well ring. `blink` = 1 s cycle, lit for the first half.
pub fn led(ui: &egui::Ui, cv: &Canvas, cx: f32, cy: f32, on: bool, decay: bool, blink: bool) {
    let lit = on && (!blink || ui.input(|i| i.time).fract() < 0.5);
    cv.circle(cx, cy, 6.0, WELL);
    if lit {
        cv.glow_circle(cx, cy, 4.0, if decay { DECAY } else { ACID });
    } else {
        cv.circle(cx, cy, 4.0, if decay { DECAY_DEEP } else { ACID_DIM });
    }
}

/// Sets a momentary bool parameter while `down` (plan.md B10).
fn momentary(
    setter: &ParamSetter,
    st: &mut UiState,
    param: &BoolParam,
    id: &'static str,
    down: bool,
) {
    if down && st.held_id != Some(id) {
        st.held_id = Some(id);
        setter.begin_set_parameter(param);
        setter.set_parameter(param, true);
    } else if !down && st.held_id == Some(id) {
        setter.set_parameter(param, false);
        setter.end_set_parameter(param);
        st.held_id = None;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FootMode {
    Toggle,
    Momentary,
}

/// Footswitch centred at (cx, cy). Normal: well 52 px, cap 40 px. Small: well 34 px, cap 26 px.
pub fn footswitch(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    param: &BoolParam,
    id: &'static str,
    cx: f32,
    cy: f32,
    small: bool,
    decay: bool,
    mode: FootMode,
    lit: bool,
) {
    let (outer, cap) = if small { (17.0, 13.0) } else { (26.0, 20.0) };
    let rect = cv.r(cx - outer, cy - outer, outer * 2.0, outer * 2.0);
    let resp = ui.interact(rect, Id::new(("hm_foot", id)), Sense::click());
    let down = resp.is_pointer_button_down_on();
    match mode {
        FootMode::Toggle => {
            if resp.clicked() {
                let v = !param.value();
                setter.begin_set_parameter(param);
                setter.set_parameter(param, v);
                setter.end_set_parameter(param);
            }
        }
        FootMode::Momentary => momentary(setter, st, param, id, down),
    }
    let tone = if decay { DECAY } else { ACID };
    if lit {
        cv.circle(cx, cy, outer + 4.0, tone.gamma_multiply(0.25));
    }
    cv.circle(cx, cy, outer, WELL);
    cv.ring(cx, cy, outer - 0.5, 1.0, Color32::BLACK);
    if lit {
        cv.ring(cx, cy, outer + 0.5, 1.0, tone);
    }
    let (ccy, cr) = if down {
        (cy + 1.0, cap * 0.96)
    } else {
        (cy, cap)
    };
    cv.circle(cx, ccy + 2.0, cr + 0.5, Color32::from_black_alpha(200));
    cv.circle(cx, ccy, cr, DECK);
    cv.circle(cx, ccy, cr * 0.88, PANEL);
    cv.circle(cx, ccy, cr * 0.48, DECK);
    cv.ring(cx, ccy, cr - 0.5, 1.0, LINE_STRONG);
    cv.ring(cx, ccy, cr * 0.4, 1.0, LINE);
    cv.arc(
        cx,
        ccy,
        cr - 1.5,
        -50.0,
        50.0,
        1.0,
        Color32::from_white_alpha(36),
    );
    let _ = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
}

/// Width of the selector box (2 px padding, 2 px gaps, options 14 px + text).
pub fn selector_width(cv: &Canvas, labels: &[&str]) -> f32 {
    4.0 + labels
        .iter()
        .map(|l| 14.0 + cv.text_width(l, Style::Choice))
        .sum::<f32>()
        + 2.0 * labels.len().saturating_sub(1) as f32
}

/// Selector: optional LABEL at `top`, then the 22 px box 16 px lower. Returns the box width.
pub fn selector<T: Enum + PartialEq + Copy + 'static>(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    param: &EnumParam<T>,
    id: &'static str,
    options: &[(T, &str)],
    x: f32,
    top: f32,
    label: Option<&str>,
    print: Color32,
    disabled: bool,
) -> f32 {
    let labels: Vec<&str> = options.iter().map(|(_, l)| *l).collect();
    let w = selector_width(cv, &labels);
    let by = match label {
        Some(text) => {
            cv.text(
                x,
                top,
                Align2::LEFT_TOP,
                text,
                Style::Label,
                dim(print, disabled),
            );
            top + 16.0
        }
        None => top,
    };
    cv.well(x, by, w, 22.0, 4.0);
    cv.stroke_rect(
        x + 1.0,
        by + 1.0,
        w - 2.0,
        20.0,
        3.0,
        1.0,
        dim(LINE_STRONG, disabled),
    );
    let current = param.value();
    let mut ox = x + 2.0;
    for (value, text) in options {
        let ow = 14.0 + cv.text_width(text, Style::Choice);
        let rect = cv.r(ox, by + 2.0, ow, 18.0);
        let sense = if disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let resp = ui.interact(rect, Id::new(("hm_sel", id, *text)), sense);
        let on = *value == current;
        if resp.clicked() && !on && !disabled {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, *value);
            setter.end_set_parameter(param);
        }
        let hover = resp.hovered() && !disabled;
        if on {
            if !disabled {
                cv.fill_rect(ox - 2.0, by, ow + 4.0, 22.0, 4.0, ACID.gamma_multiply(0.25));
            }
            cv.fill_rect(ox, by + 2.0, ow, 18.0, 2.0, dim(ACID, disabled));
        } else if hover {
            cv.fill_rect(ox, by + 2.0, ow, 18.0, 2.0, ACID_DEEP);
        }
        let ink = if on {
            ON_ACID
        } else if hover {
            ACID
        } else {
            dim(INK_MUTED, disabled)
        };
        cv.text(
            ox + ow / 2.0,
            by + 11.0,
            Align2::CENTER_CENTER,
            text,
            Style::Choice,
            ink,
        );
        ox += ow + 2.0;
    }
    w
}

pub fn toggle_width(cv: &Canvas, text: &str) -> f32 {
    26.0 + cv.text_width(text, Style::Choice)
}

/// Toggle (22 px high) at top-left (x, y). `width` lets a column stretch it.
pub fn toggle(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    param: &BoolParam,
    id: &'static str,
    x: f32,
    y: f32,
    width: f32,
    text: &str,
    decay: bool,
) {
    let rect = cv.r(x, y, width, 22.0);
    let resp = ui.interact(rect, Id::new(("hm_toggle", id)), Sense::click());
    let on = param.value();
    if resp.clicked() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, !on);
        setter.end_set_parameter(param);
    }
    cv.well(x, y, width, 22.0, 2.0);
    cv.stroke_rect(x + 1.0, y + 1.0, width - 2.0, 20.0, 1.0, 1.0, LINE_STRONG);
    let tone = if decay { DECAY } else { ACID };
    if on {
        cv.glow_circle(x + 10.0, y + 11.0, 3.0, tone);
    } else {
        cv.circle(x + 10.0, y + 11.0, 3.0, ACID_DIM);
    }
    let ink = if on || resp.hovered() { INK } else { INK_MUTED };
    cv.text(
        x + 18.0,
        y + 11.0,
        Align2::LEFT_CENTER,
        text,
        Style::Choice,
        ink,
    );
}

pub fn push_width(cv: &Canvas, text: &str) -> f32 {
    32.0 + cv.text_width(text, Style::Choice)
}

/// Momentary push button (22 px high) with a 12 px icon, top-left at (x, y).
pub fn push_button(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    param: &BoolParam,
    id: &'static str,
    x: f32,
    y: f32,
    text: &str,
    which: Icon,
) {
    let w = push_width(cv, text);
    let rect = cv.r(x, y, w, 22.0);
    let resp = ui.interact(rect, Id::new(("hm_push", id)), Sense::click());
    let down = resp.is_pointer_button_down_on();
    momentary(setter, st, param, id, down);
    cv.well(x, y, w, 22.0, 2.0);
    cv.stroke_rect(
        x + 1.0,
        y + 1.0,
        w - 2.0,
        20.0,
        1.0,
        1.0,
        if down { ACID } else { LINE_STRONG },
    );
    let ink = if down {
        ACID
    } else if resp.hovered() {
        INK
    } else {
        INK_MUTED
    };
    icon(cv, which, x + 7.0, y + 5.0, 12.0, ink);
    cv.text(
        x + 24.0,
        y + 11.0,
        Align2::LEFT_CENTER,
        text,
        Style::Choice,
        ink,
    );
}
