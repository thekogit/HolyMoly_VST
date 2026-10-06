//! Stereo 1x/2x/4x oversampler with constant 24-sample latency (plan.md B5, B6).

use std::f64::consts::PI;

/// Latency in base-rate samples at every factor.
pub const OS_LATENCY: usize = 24;

const TAPS_A: usize = 39;
const TAPS_B: usize = 21;

/// Windowed-sinc halfband lowpass (cutoff = quarter of the high rate), normalised so that
/// each polyphase branch has DC gain 0.5 exactly.
fn design_halfband(n: usize) -> Vec<f32> {
    let m = (n - 1) / 2;
    let mut h = vec![0.0f64; n];
    for i in 0..n {
        let k = i as f64 - m as f64;
        let x = PI * 0.5 * k;
        let sinc = if i == m { 1.0 } else { x.sin() / x };
        let t = i as f64 / (n - 1) as f64;
        let w = 0.42 - 0.5 * (2.0 * PI * t).cos() + 0.08 * (4.0 * PI * t).cos();
        h[i] = 0.5 * sinc * w;
    }
    let mut other_sum = 0.0;
    for i in 0..n {
        if (i as i64 - m as i64) % 2 != 0 {
            other_sum += h[i];
        }
    }
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        let k = i as i64 - m as i64;
        out[i] = if k == 0 {
            0.5
        } else if k % 2 == 0 {
            0.0
        } else {
            (h[i] * 0.5 / other_sum) as f32
        };
    }
    out
}

struct Fir {
    taps: Vec<f32>,
    hist: Vec<f32>,
    pos: usize,
}

impl Fir {
    fn new(taps: Vec<f32>) -> Self {
        let n = taps.len();
        Self {
            taps,
            hist: vec![0.0; n],
            pos: 0,
        }
    }

    fn reset(&mut self) {
        self.hist.fill(0.0);
        self.pos = 0;
    }

    #[inline]
    fn push(&mut self, x: f32) {
        self.pos += 1;
        if self.pos == self.hist.len() {
            self.pos = 0;
        }
        self.hist[self.pos] = x;
    }

    #[inline]
    fn out(&self) -> f32 {
        let n = self.hist.len();
        let mut acc = 0.0f32;
        let mut idx = self.pos;
        for &t in &self.taps {
            acc += t * self.hist[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }
        acc
    }
}

/// One channel of one 2x stage.
struct Stage {
    up: Fir,
    down: Fir,
}

impl Stage {
    fn new(taps: usize) -> Self {
        let t = design_halfband(taps);
        Self {
            up: Fir::new(t.clone()),
            down: Fir::new(t),
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }

    #[inline]
    fn upsample(&mut self, x: f32) -> [f32; 2] {
        self.up.push(2.0 * x);
        let a = self.up.out();
        self.up.push(0.0);
        let b = self.up.out();
        [a, b]
    }

    /// Output is taken on the even phase so the delay stays an integer.
    #[inline]
    fn downsample(&mut self, a: f32, b: f32) -> f32 {
        self.down.push(a);
        let y = self.down.out();
        self.down.push(b);
        y
    }
}

struct Pad {
    buf: [f32; 32],
    pos: usize,
    delay: usize,
}

impl Pad {
    fn new() -> Self {
        Self {
            buf: [0.0; 32],
            pos: 0,
            delay: 0,
        }
    }

    fn reset(&mut self, delay: usize) {
        self.buf = [0.0; 32];
        self.pos = 0;
        self.delay = delay.min(31);
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        if self.delay == 0 {
            return x;
        }
        self.buf[self.pos] = x;
        let y = self.buf[(self.pos + 32 - self.delay) % 32];
        self.pos = (self.pos + 1) % 32;
        y
    }
}

struct OsChannel {
    a: Stage,
    b: Stage,
    pad: Pad,
}

impl OsChannel {
    fn new() -> Self {
        Self {
            a: Stage::new(TAPS_A),
            b: Stage::new(TAPS_B),
            pad: Pad::new(),
        }
    }
}

pub struct Oversampler {
    ch: [OsChannel; 2],
    factor: usize,
}

impl Default for Oversampler {
    fn default() -> Self {
        Self::new()
    }
}

impl Oversampler {
    /// Allocates. NOT for the audio thread.
    pub fn new() -> Self {
        let mut os = Self {
            ch: [OsChannel::new(), OsChannel::new()],
            factor: 4,
        };
        os.reset();
        os
    }

    pub fn factor(&self) -> usize {
        self.factor
    }

    pub fn reset(&mut self) {
        let delay = match self.factor {
            1 => 24,
            2 => 5,
            _ => 0,
        };
        for c in self.ch.iter_mut() {
            c.a.reset();
            c.b.reset();
            c.pad.reset(delay);
        }
    }

    /// Processes `left`/`right` in place. `f` runs once per oversampled stereo frame.
    pub fn process<F>(&mut self, factor: usize, left: &mut [f32], right: &mut [f32], mut f: F)
    where
        F: FnMut(f32, f32) -> (f32, f32),
    {
        let factor = match factor {
            1 => 1,
            2 => 2,
            _ => 4,
        };
        if factor != self.factor {
            self.factor = factor;
            self.reset();
        }
        let n = left.len().min(right.len());
        let [cl, cr] = &mut self.ch;
        for i in 0..n {
            let (yl, yr) = if factor == 1 {
                f(left[i], right[i])
            } else if factor == 2 {
                let ul = cl.a.upsample(left[i]);
                let ur = cr.a.upsample(right[i]);
                let (l0, r0) = f(ul[0], ur[0]);
                let (l1, r1) = f(ul[1], ur[1]);
                (cl.a.downsample(l0, l1), cr.a.downsample(r0, r1))
            } else {
                let ul = cl.a.upsample(left[i]);
                let ur = cr.a.upsample(right[i]);
                let mut ml = [0.0f32; 2];
                let mut mr = [0.0f32; 2];
                for k in 0..2 {
                    let ql = cl.b.upsample(ul[k]);
                    let qr = cr.b.upsample(ur[k]);
                    let (l0, r0) = f(ql[0], qr[0]);
                    let (l1, r1) = f(ql[1], qr[1]);
                    ml[k] = cl.b.downsample(l0, l1);
                    mr[k] = cr.b.downsample(r0, r1);
                }
                (cl.a.downsample(ml[0], ml[1]), cr.a.downsample(mr[0], mr[1]))
            };
            left[i] = cl.pad.process(yl);
            right[i] = cr.pad.process(yr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;

    fn run(factor: usize, input: &[f32], mut f: impl FnMut(f32, f32) -> (f32, f32)) -> Vec<f32> {
        let mut os = Oversampler::new();
        let mut l = input.to_vec();
        let mut r = input.to_vec();
        for (cl, cr) in l.chunks_mut(32).zip(r.chunks_mut(32)) {
            os.process(factor, cl, cr, &mut f);
        }
        l
    }

    #[test]
    fn latency_is_24_at_every_factor() {
        for factor in [1usize, 2, 4] {
            let mut x = vec![0.0f32; 256];
            x[10] = 1.0;
            let y = run(factor, &x, |a, b| (a, b));
            assert_eq!(argmax_abs(&y), 10 + OS_LATENCY, "factor {factor}");
        }
    }

    #[test]
    fn dc_gain_is_unity() {
        for factor in [1usize, 2, 4] {
            let x = vec![0.5f32; 512];
            let y = run(factor, &x, |a, b| (a, b));
            assert!((y[511] - 0.5).abs() < 1e-3, "factor {factor}: {}", y[511]);
        }
    }

    #[test]
    fn passband_is_flat_at_1khz() {
        let sr = 48000.0;
        let x = sine(1000.0, sr, 4800, 0.5);
        for factor in [2usize, 4] {
            let y = run(factor, &x, |a, b| (a, b));
            let a = tone_amplitude(&y[480..4800], 1000.0, sr);
            assert!((db(a) - db(0.5)).abs() < 0.3, "factor {factor}: {a}");
        }
    }

    #[test]
    fn oversampling_reduces_aliasing() {
        let sr = 48000.0;
        let x = sine(15000.0, sr, 4800, 0.9);
        let cube = |a: f32, b: f32| (a * a * a, b * b * b);
        let y1 = run(1, &x, cube);
        let y4 = run(4, &x, cube);
        let alias1 = tone_amplitude(&y1[480..4800], 3000.0, sr);
        let alias4 = tone_amplitude(&y4[480..4800], 3000.0, sr);
        assert!(
            alias1 > 0.1,
            "sanity: alias at 1x should be strong, got {alias1}"
        );
        assert!(db(alias4) < db(alias1) - 30.0, "1x {alias1}, 4x {alias4}");
    }

    #[test]
    fn reset_clears_history() {
        let mut os = Oversampler::new();
        let mut l = noise(256, 0.5, 1);
        let mut r = noise(256, 0.5, 2);
        os.process(4, &mut l, &mut r, |a, b| (a, b));
        os.reset();
        let mut zl = vec![0.0f32; 256];
        let mut zr = vec![0.0f32; 256];
        os.process(4, &mut zl, &mut zr, |a, b| (a, b));
        assert_eq!(peak(&zl), 0.0);
        assert_eq!(peak(&zr), 0.0);
    }
}
