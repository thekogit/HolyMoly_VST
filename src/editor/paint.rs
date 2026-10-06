//! Drawing helpers (plan.md C2 glow/disabled rules, C3 text styles, D4.0 erratum).
//! All coordinates are EDITOR coordinates (0..1120, 0..760); `Canvas` adds the window origin.
//! Only egui calls that are stable across versions are used here (plan.md Part E, E-E2).

use crate::editor::theme::*;
use nih_plug_egui::egui::{
    self, pos2, vec2, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Wordmark,
    PedalName,
    Note,
    Label,
    Value,
    Choice,
    Readout,
    Stage,
}

impl Style {
    pub fn family(self) -> &'static str {
        match self {
            Style::Wordmark => "display900",
            Style::PedalName | Style::Note => "display800",
            Style::Label => "mono700",
            Style::Value | Style::Stage => "mono500",
            Style::Choice | Style::Readout => "mono600",
        }
    }

    pub fn size(self) -> f32 {
        match self {
            Style::Wordmark => 40.0,
            Style::PedalName => 26.0,
            Style::Note => 52.0,
            Style::Readout => 13.0,
            _ => 10.0,
        }
    }

    /// Letter spacing in em.
    pub fn tracking(self) -> f32 {
        match self {
            Style::Wordmark => 0.02,
            Style::PedalName => 0.03,
            Style::Note => 0.0,
            Style::Label => 0.08,
            Style::Value => 0.02,
            Style::Choice => 0.06,
            Style::Readout => 0.04,
            Style::Stage => 0.10,
        }
    }

    /// plan.md D4.0: only LABEL, OPTION (Choice) and PEDAL_NAME are uppercased.
    pub fn upper(self) -> bool {
        matches!(self, Style::Label | Style::Choice | Style::PedalName)
    }

    pub fn font(self) -> FontId {
        FontId::new(self.size(), FontFamily::Name(self.family().into()))
    }
}

/// Point at angle `deg` (clockwise from 12 o'clock) on a circle. Same as `polar` in the reference.
pub fn polar(cx: f32, cy: f32, r: f32, deg: f32) -> (f32, f32) {
    let t = deg.to_radians();
    (cx + r * t.sin(), cy - r * t.cos())
}

/// Disabled rule (plan.md C2).
pub fn dim(c: Color32, disabled: bool) -> Color32 {
    if disabled {
        c.gamma_multiply(0.32)
    } else {
        c
    }
}

/// Outline points of a rounded rectangle, clockwise, 28 points.
pub fn rounded_rect_points(rect: Rect, radius: f32) -> Vec<Pos2> {
    let r = radius.min(rect.width().min(rect.height()) / 2.0);
    let r = if r < 0.0 { 0.0 } else { r };
    let corners = [
        (rect.right() - r, rect.top() + r, -90.0f32),
        (rect.right() - r, rect.bottom() - r, 0.0),
        (rect.left() + r, rect.bottom() - r, 90.0),
        (rect.left() + r, rect.top() + r, 180.0),
    ];
    let mut pts = Vec::with_capacity(28);
    for (cx, cy, start) in corners {
        for k in 0..7 {
            let a = (start + 15.0 * k as f32).to_radians();
            pts.push(pos2(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    pts
}

#[derive(Clone)]
pub struct Canvas {
    pub painter: Painter,
    pub origin: Vec2,
}

impl Canvas {
    pub fn new(ui: &egui::Ui) -> Self {
        Self {
            painter: ui.painter().clone(),
            origin: ui.max_rect().min.to_vec2(),
        }
    }

    pub fn p(&self, x: f32, y: f32) -> Pos2 {
        pos2(x, y) + self.origin
    }

    pub fn r(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.p(x, y), vec2(w, h))
    }

    /// A canvas that only draws inside the given rectangle.
    pub fn clipped(&self, x: f32, y: f32, w: f32, h: f32) -> Canvas {
        Canvas {
            painter: self.painter.with_clip_rect(self.r(x, y, w, h)),
            origin: self.origin,
        }
    }

    pub fn fill_rect(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color32) {
        self.painter.rect_filled(self.r(x, y, w, h), radius, color);
    }

    pub fn stroke_rect(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radius: f32,
        width: f32,
        color: Color32,
    ) {
        let half = width / 2.0;
        let rect = self.r(x + half, y + half, w - width, h - width);
        let pts = rounded_rect_points(rect, (radius - half).max(0.0));
        self.painter
            .add(Shape::closed_line(pts, Stroke::new(width, color)));
    }

    pub fn circle(&self, cx: f32, cy: f32, r: f32, color: Color32) {
        self.painter.circle_filled(self.p(cx, cy), r, color);
    }

    pub fn ring(&self, cx: f32, cy: f32, r: f32, width: f32, color: Color32) {
        self.painter
            .circle_stroke(self.p(cx, cy), r, Stroke::new(width, color));
    }

    /// Glow rule (plan.md C2): a larger copy at 25 % opacity, then the shape.
    pub fn glow_circle(&self, cx: f32, cy: f32, r: f32, color: Color32) {
        self.circle(cx, cy, r + 3.0, color.gamma_multiply(0.25));
        self.circle(cx, cy, r, color);
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color32) {
        self.painter
            .line_segment([self.p(x1, y1), self.p(x2, y2)], Stroke::new(width, color));
    }

    pub fn glow_line(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color32) {
        self.line(x1, y1, x2, y2, width + 3.0, color.gamma_multiply(0.25));
        self.line(x1, y1, x2, y2, width, color);
    }

    pub fn polyline(&self, pts: &[(f32, f32)], width: f32, color: Color32) {
        if pts.len() < 2 {
            return;
        }
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.p(*x, *y)).collect();
        self.painter.add(Shape::line(v, Stroke::new(width, color)));
    }

    /// Filled CONVEX polygon.
    pub fn polygon(&self, pts: &[(f32, f32)], color: Color32) {
        if pts.len() < 3 {
            return;
        }
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.p(*x, *y)).collect();
        self.painter
            .add(Shape::convex_polygon(v, color, Stroke::NONE));
    }

    /// Arc from `a0` to `a1` degrees (clockwise from 12 o'clock).
    pub fn arc(&self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, width: f32, color: Color32) {
        let (a0, a1) = if a1 < a0 { (a1, a0) } else { (a0, a1) };
        if a1 - a0 < 0.5 {
            return;
        }
        let n = (((a1 - a0) / 4.0).ceil() as usize).max(2);
        let pts: Vec<(f32, f32)> = (0..=n)
            .map(|k| polar(cx, cy, r, a0 + (a1 - a0) * k as f32 / n as f32))
            .collect();
        self.polyline(&pts, width, color);
    }

    pub fn glow_arc(&self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, width: f32, color: Color32) {
        self.arc(cx, cy, r, a0, a1, width + 3.0, color.gamma_multiply(0.25));
        self.arc(cx, cy, r, a0, a1, width, color);
    }

    /// Dashed arc; `dash` and `gap` in px along the circle.
    pub fn dashed_arc(
        &self,
        cx: f32,
        cy: f32,
        r: f32,
        a0: f32,
        a1: f32,
        width: f32,
        color: Color32,
        dash: f32,
        gap: f32,
    ) {
        let deg_per_px = 180.0 / (std::f32::consts::PI * r.max(1.0));
        let mut a = a0.min(a1);
        let end = a0.max(a1);
        while a < end {
            let b = (a + dash * deg_per_px).min(end);
            let (x0, y0) = polar(cx, cy, r, a);
            let (x1, y1) = polar(cx, cy, r, b);
            self.line(x0, y0, x1, y1, width, color);
            a = b + gap * deg_per_px;
        }
    }

    /// Diagonal "/" stripes (CSS repeating-linear-gradient(-45deg, color 0 band, transparent band period)),
    /// clipped to the rectangle. `band` and `period` are measured across the stripes.
    pub fn stripes(&self, x: f32, y: f32, w: f32, h: f32, band: f32, period: f32, color: Color32) {
        let c = self.clipped(x, y, w, h);
        let p = period * std::f32::consts::SQRT_2;
        let b = band * std::f32::consts::SQRT_2;
        let first = -((h / p).ceil() as i32) - 1;
        let last = (w / p).ceil() as i32 + 1;
        for k in first..=last {
            let x0 = x + k as f32 * p;
            c.polygon(
                &[(x0, y + h), (x0 + b, y + h), (x0 + b + h, y), (x0 + h, y)],
                color,
            );
        }
    }

    /// `--shadow-pedal` approximation; draw BEFORE the body.
    pub fn pedal_shadow(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) {
        self.fill_rect(
            x - 1.0,
            y + 6.0,
            w + 2.0,
            h + 2.0,
            radius + 2.0,
            Color32::from_black_alpha(70),
        );
        self.fill_rect(
            x,
            y + 2.0,
            w,
            h + 1.0,
            radius,
            Color32::from_black_alpha(140),
        );
    }

    /// Top highlight and dark inner border of `--shadow-pedal`; draw AFTER the body.
    pub fn pedal_bevel(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) {
        self.stroke_rect(x, y, w, h, radius, 1.0, Color32::from_black_alpha(89));
        self.line(
            x + radius,
            y + 1.0,
            x + w - radius,
            y + 1.0,
            1.0,
            Color32::from_white_alpha(31),
        );
    }

    /// `--shadow-well`: near-black inset box with a black edge.
    pub fn well(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) {
        self.fill_rect(x, y, w, h, radius, WELL);
        self.stroke_rect(x, y, w, h, radius, 1.0, Color32::BLACK);
    }

    fn glyph_widths(&self, s: &str, font: &FontId) -> (Vec<f32>, f32) {
        let mut widths = Vec::with_capacity(s.len());
        let mut height = 0.0f32;
        for ch in s.chars() {
            let g = self
                .painter
                .layout_no_wrap(ch.to_string(), font.clone(), Color32::WHITE);
            widths.push(g.size().x);
            height = height.max(g.size().y);
        }
        (widths, height)
    }

    fn prepared(&self, text: &str, style: Style) -> String {
        if style.upper() {
            text.to_uppercase()
        } else {
            text.to_owned()
        }
    }

    /// Width of `text` in `style`, including letter spacing.
    pub fn text_width(&self, text: &str, style: Style) -> f32 {
        let s = self.prepared(text, style);
        let (widths, _) = self.glyph_widths(&s, &style.font());
        let track = style.tracking() * style.size();
        widths.iter().sum::<f32>() + track * widths.len().saturating_sub(1) as f32
    }

    /// Draws `text` with letter spacing. (x, y) is the anchor point. Returns the drawn rect (window coords).
    pub fn text(
        &self,
        x: f32,
        y: f32,
        anchor: Align2,
        text: &str,
        style: Style,
        color: Color32,
    ) -> Rect {
        let s = self.prepared(text, style);
        let font = style.font();
        let (widths, height) = self.glyph_widths(&s, &font);
        let track = style.tracking() * style.size();
        let total = widths.iter().sum::<f32>() + track * widths.len().saturating_sub(1) as f32;
        let rect = anchor.anchor_size(self.p(x, y), vec2(total, height));
        let mut cx = rect.min.x;
        for (ch, w) in s.chars().zip(widths.iter()) {
            self.painter.text(
                pos2(cx, rect.min.y),
                Align2::LEFT_TOP,
                ch,
                font.clone(),
                color,
            );
            cx += w + track;
        }
        rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polar_matches_the_reference() {
        let (x, y) = polar(0.0, 0.0, 10.0, 0.0);
        assert!(x.abs() < 1e-5 && (y + 10.0).abs() < 1e-5);
        let (x, y) = polar(0.0, 0.0, 10.0, 90.0);
        assert!((x - 10.0).abs() < 1e-5 && y.abs() < 1e-4);
    }

    #[test]
    fn rounded_rect_has_28_points_inside_the_rect() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 50.0));
        let pts = rounded_rect_points(rect, 6.0);
        assert_eq!(pts.len(), 28);
        for p in pts {
            assert!(p.x >= -0.01 && p.x <= 100.01 && p.y >= -0.01 && p.y <= 50.01);
        }
    }
}
