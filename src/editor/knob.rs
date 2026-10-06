//! Knob widget (reference: `Knob` in holy-moly-editor.html).

use crate::editor::paint::{dim, polar, Canvas, Style};
use crate::editor::theme::*;
use crate::editor::UiState;
use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui::{self, Align2, Color32, Id, Mesh, Sense, Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnobSize {
    Sm,
    Md,
    Lg,
}

impl KnobSize {
    pub fn cap(self) -> f32 {
        match self {
            KnobSize::Sm => 28.0,
            KnobSize::Md => 34.0,
            KnobSize::Lg => 42.0,
        }
    }

    pub fn dial(self) -> f32 {
        self.cap() + 18.0
    }

    /// Dial + 3 + label 12 + 3 + value window 14 (plan.md C4.3).
    pub fn height(self) -> f32 {
        self.dial() + 32.0
    }
}

/// Every knob slot is 64 px wide (plan.md B13).
pub const KNOB_SLOT_W: f32 = 64.0;

#[derive(Clone, Copy, Debug)]
pub struct KnobSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub size: KnobSize,
    pub bipolar: bool,
    /// Normalized value where the danger zone starts.
    pub danger: Option<f32>,
    /// Number of positions for stepped knobs.
    pub steps: Option<usize>,
    pub disabled: bool,
    /// Label colour (the pedal's print colour).
    pub print: Color32,
}

impl KnobSpec {
    pub const fn md(id: &'static str, label: &'static str, print: Color32) -> Self {
        Self {
            id,
            label,
            size: KnobSize::Md,
            bipolar: false,
            danger: None,
            steps: None,
            disabled: false,
            print,
        }
    }
}

pub fn angle_of(norm: f32) -> f32 {
    -135.0 + 270.0 * norm
}

/// Snaps to the nearest step for stepped knobs.
pub fn snap(norm: f32, steps: Option<usize>) -> f32 {
    match steps {
        Some(n) if n > 1 => {
            let m = (n - 1) as f32;
            (norm * m).round() / m
        }
        _ => norm,
    }
}

fn value_arc(
    cv: &Canvas,
    cx: f32,
    cy: f32,
    r: f32,
    a0: f32,
    a1: f32,
    color: Color32,
    disabled: bool,
) {
    if disabled {
        cv.arc(cx, cy, r, a0, a1, 2.5, color);
    } else {
        cv.glow_arc(cx, cy, r, a0, a1, 2.5, color);
    }
}

fn draw_cap(cv: &Canvas, cx: f32, cy: f32, cap: f32, angle: f32, disabled: bool) {
    let r = cap / 2.0;
    cv.circle(cx, cy + 2.0, r + 0.5, Color32::from_black_alpha(160));
    let mut mesh = Mesh::default();
    let c = cv.p(cx, cy);
    for k in 0..48 {
        let col = dim(if k % 2 == 0 { PANEL } else { DECK }, disabled);
        let a0 = angle + 7.5 * k as f32;
        let (x0, y0) = polar(cx, cy, r, a0);
        let (x1, y1) = polar(cx, cy, r, a0 + 7.5);
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(c, col);
        mesh.colored_vertex(cv.p(x0, y0), col);
        mesh.colored_vertex(cv.p(x1, y1), col);
        mesh.add_triangle(i, i + 1, i + 2);
    }
    cv.painter.add(Shape::mesh(mesh));
    cv.ring(cx, cy, r - 0.5, 1.0, dim(LINE_STRONG, disabled));
    cv.circle(cx, cy, r - 4.0, dim(PANEL, disabled));
    cv.arc(
        cx,
        cy,
        r - 4.5,
        -60.0,
        60.0,
        1.0,
        Color32::from_white_alpha(20),
    );
}

/// Draws and handles one knob. `cx` = slot centre x, `top` = top of the dial box (editor coords).
pub fn knob<P: Param>(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    param: &P,
    cx: f32,
    top: f32,
    spec: KnobSpec,
) {
    let dial = spec.size.dial();
    let cap = spec.size.cap();
    let (dx, dy) = (cx, top + dial / 2.0);
    let rect = cv.r(cx - dial / 2.0, top, dial, dial);
    let sense = if spec.disabled {
        Sense::hover()
    } else {
        Sense::click_and_drag()
    };
    let resp = ui.interact(rect, Id::new(("hm_knob", spec.id)), sense);
    let mut norm = param.unmodulated_normalized_value();

    if !spec.disabled {
        if resp.double_clicked() {
            let d = param.default_normalized_value();
            setter.begin_set_parameter(param);
            setter.set_parameter_normalized(param, d);
            setter.end_set_parameter(param);
            norm = d;
        } else if resp.dragged() {
            if st.drag_id != Some(spec.id) {
                st.drag_id = Some(spec.id);
                st.drag_norm = norm;
                setter.begin_set_parameter(param);
            }
            let fine = ui.input(|i| i.modifiers.shift);
            let px = if fine { 2000.0 } else { 200.0 };
            st.drag_norm = (st.drag_norm - resp.drag_delta().y / px).clamp(0.0, 1.0);
            norm = snap(st.drag_norm, spec.steps);
            setter.set_parameter_normalized(param, norm);
        } else if st.drag_id == Some(spec.id) {
            setter.end_set_parameter(param);
            st.drag_id = None;
        }
        let _ = resp.on_hover_cursor(egui::CursorIcon::ResizeVertical);
    }

    let off = spec.disabled;
    let hot = matches!(spec.danger, Some(d) if norm > d + 1e-4);
    let accent = if hot { DECAY } else { ACID };

    cv.circle(dx, dy, dial / 2.0, WELL);
    cv.ring(dx, dy, dial / 2.0 - 0.5, 1.0, Color32::BLACK);

    let r = cap / 2.0 + 4.5;
    let av = angle_of(norm);
    match spec.steps {
        Some(n) if n > 1 => {
            cv.arc(dx, dy, r, -135.0, 135.0, 2.5, dim(LINE, off));
            let lit = (norm * (n - 1) as f32).round() as usize;
            for i in 0..n {
                let (x, y) = polar(dx, dy, r, angle_of(i as f32 / (n - 1) as f32));
                if i == lit {
                    if off {
                        cv.circle(x, y, 2.2, dim(ACID, off));
                    } else {
                        cv.glow_circle(x, y, 2.2, ACID);
                    }
                } else {
                    cv.circle(x, y, 1.3, dim(INK_FAINT, off));
                }
            }
        }
        _ => {
            let base = if spec.bipolar { 0.0 } else { -135.0 };
            match spec.danger {
                Some(d) => {
                    let ad = angle_of(d);
                    cv.arc(dx, dy, r, -135.0, ad, 2.5, dim(LINE, off));
                    cv.dashed_arc(dx, dy, r, ad, 135.0, 2.5, dim(DECAY, off), 1.5, 2.5);
                    value_arc(cv, dx, dy, r, base, av.min(ad), dim(ACID, off), off);
                    if av > ad {
                        value_arc(cv, dx, dy, r, ad, av, dim(DECAY, off), off);
                    }
                }
                None => {
                    cv.arc(dx, dy, r, -135.0, 135.0, 2.5, dim(LINE, off));
                    value_arc(cv, dx, dy, r, base, av, dim(ACID, off), off);
                }
            }
            if spec.bipolar {
                let (x, y) = polar(dx, dy, r + 3.2, 0.0);
                cv.circle(x, y, 1.0, dim(INK_FAINT, off));
            }
        }
    }

    draw_cap(cv, dx, dy, cap, av, off);

    let outer = cap / 2.0 - 2.0;
    let inner = outer - 0.38 * cap;
    let (x0, y0) = polar(dx, dy, outer, av);
    let (x1, y1) = polar(dx, dy, inner, av);
    if off {
        cv.line(x0, y0, x1, y1, 2.0, dim(accent, off));
    } else {
        cv.glow_line(x0, y0, x1, y1, 2.0, accent);
    }

    let label_top = top + dial + 3.0;
    cv.text(
        cx,
        label_top,
        Align2::CENTER_TOP,
        spec.label,
        Style::Label,
        dim(spec.print, off),
    );

    let text = param.normalized_value_to_string(norm, true);
    let w = (cv.text_width(&text, Style::Value) + 8.0).max(48.0);
    let wy = label_top + 15.0;
    cv.well(cx - w / 2.0, wy, w, 14.0, 4.0);
    if st.drag_id == Some(spec.id) {
        cv.stroke_rect(cx - w / 2.0, wy, w, 14.0, 4.0, 1.0, ACID);
    }
    cv.text(
        cx,
        wy + 7.0,
        Align2::CENTER_CENTER,
        &text,
        Style::Value,
        dim(accent, off),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_match_the_reference() {
        assert_eq!(angle_of(0.0), -135.0);
        assert_eq!(angle_of(0.5), 0.0);
        assert_eq!(angle_of(1.0), 135.0);
    }

    #[test]
    fn sizes_match_c4_3() {
        assert_eq!(KnobSize::Sm.height(), 78.0);
        assert_eq!(KnobSize::Md.height(), 84.0);
        assert_eq!(KnobSize::Lg.height(), 92.0);
    }

    #[test]
    fn stepped_knobs_snap() {
        assert_eq!(snap(0.49, Some(3)), 0.5);
        assert_eq!(snap(0.49, None), 0.49);
        assert_eq!(snap(0.5, Some(13)), 0.5);
    }
}
