//! Shared DSP building blocks. Plain Rust, no nih-plug.

use std::f32::consts::PI;

/// The chain updates parameters every SUB samples (plan.md B4).
pub const SUB: usize = 32;

pub fn db_to_gain(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.abs().max(1e-12).log10()
}

/// Blend from `dry` (amount 0) to `wet` (amount 1).
#[inline]
pub fn mix(dry: f32, wet: f32, amount: f32) -> f32 {
    dry + (wet - dry) * amount
}

/// Exponential smoother. `set_time(ms)` = time to cover about 99 % of a step.
#[derive(Clone, Copy, Debug)]
pub struct Smoother {
    current: f32,
    target: f32,
    coeff: f32,
}

impl Smoother {
    pub fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            coeff: 0.0,
        }
    }

    pub fn set_time(&mut self, sample_rate: f32, ms: f32) {
        let samples = ms * 0.001 * sample_rate;
        self.coeff = if samples < 1.0 {
            0.0
        } else {
            (-4.7 / samples).exp()
        };
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn reset(&mut self, value: f32) {
        self.current = value;
        self.target = value;
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    /// Moves `n` samples toward the target. Returns (value before, value after).
    pub fn block(&mut self, n: usize) -> (f32, f32) {
        let before = self.current;
        let k = self.coeff.powi(n as i32);
        self.current = self.target + (self.current - self.target) * k;
        if (self.current - self.target).abs() < 1e-6 {
            self.current = self.target;
        }
        (before, self.current)
    }
}

/// Linear interpolation across one sub-block: `set(from, to, ticks)`, then call `step()` `ticks` times.
#[derive(Clone, Copy, Debug, Default)]
pub struct Glide {
    value: f32,
    inc: f32,
}

impl Glide {
    pub fn set(&mut self, from: f32, to: f32, ticks: usize) {
        self.value = from;
        self.inc = (to - from) / ticks.max(1) as f32;
    }

    pub fn hold(&mut self, value: f32) {
        self.value = value;
        self.inc = 0.0;
    }

    #[inline]
    pub fn step(&mut self) -> f32 {
        self.value += self.inc;
        self.value
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// On/off with a 10 ms linear crossfade (footswitches, plan.md concept "Bypass").
#[derive(Clone, Copy, Debug)]
pub struct Switch {
    value: f32,
    target: f32,
    step: f32,
}

impl Switch {
    pub fn new(on: bool) -> Self {
        let v = if on { 1.0 } else { 0.0 };
        Self {
            value: v,
            target: v,
            step: 1.0 / 480.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.step = 1.0 / (0.010 * sample_rate).max(1.0);
    }

    pub fn set(&mut self, on: bool) {
        self.target = if on { 1.0 } else { 0.0 };
    }

    pub fn reset(&mut self, on: bool) {
        self.set(on);
        self.value = self.target;
    }

    /// Moves `n` samples toward the target. Returns (value before, value after).
    pub fn block(&mut self, n: usize) -> (f32, f32) {
        let before = self.value;
        let delta = self.step * n as f32;
        self.value = if self.target > self.value {
            (self.value + delta).min(self.target)
        } else {
            (self.value - delta).max(self.target)
        };
        (before, self.value)
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_off(&self) -> bool {
        self.value == 0.0 && self.target == 0.0
    }

    pub fn is_on(&self) -> bool {
        self.value == 1.0 && self.target == 1.0
    }
}

/// First-order TPT filter. `tick` returns (lowpass, highpass); lowpass + highpass == input.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePole {
    g: f32,
    s: f32,
}

impl OnePole {
    pub fn set_cutoff(&mut self, sample_rate: f32, hz: f32) {
        let hz = hz.clamp(1.0, sample_rate * 0.49);
        let wc = (PI * hz / sample_rate).tan();
        self.g = wc / (1.0 + wc);
    }

    pub fn reset(&mut self) {
        self.s = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> (f32, f32) {
        let v = (x - self.s) * self.g;
        let lp = v + self.s;
        self.s = lp + v;
        (lp, x - lp)
    }

    #[inline]
    pub fn lowpass(&mut self, x: f32) -> f32 {
        self.tick(x).0
    }

    #[inline]
    pub fn highpass(&mut self, x: f32) -> f32 {
        self.tick(x).1
    }
}

/// RBJ biquad, transposed direct form II. Default = pass-through.
#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }
}

impl Biquad {
    fn prep(sample_rate: f32, hz: f32, q: f32) -> (f32, f32) {
        let hz = hz.clamp(10.0, sample_rate * 0.45);
        let w0 = 2.0 * PI * hz / sample_rate;
        (w0, w0.sin() / (2.0 * q.max(0.1)))
    }

    fn set(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn set_lowpass(&mut self, sample_rate: f32, hz: f32, q: f32) {
        let (w0, alpha) = Self::prep(sample_rate, hz, q);
        let c = w0.cos();
        self.set(
            (1.0 - c) / 2.0,
            1.0 - c,
            (1.0 - c) / 2.0,
            1.0 + alpha,
            -2.0 * c,
            1.0 - alpha,
        );
    }

    pub fn set_highpass(&mut self, sample_rate: f32, hz: f32, q: f32) {
        let (w0, alpha) = Self::prep(sample_rate, hz, q);
        let c = w0.cos();
        self.set(
            (1.0 + c) / 2.0,
            -(1.0 + c),
            (1.0 + c) / 2.0,
            1.0 + alpha,
            -2.0 * c,
            1.0 - alpha,
        );
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// DC blocker, corner at about 10 Hz.
#[derive(Clone, Copy, Debug)]
pub struct DcBlocker {
    r: f32,
    x1: f32,
    y1: f32,
}

impl Default for DcBlocker {
    fn default() -> Self {
        Self {
            r: 0.9987,
            x1: 0.0,
            y1: 0.0,
        }
    }
}

impl DcBlocker {
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.r = 1.0 - 2.0 * PI * 10.0 / sample_rate;
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + self.r * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

/// Mono delay line with linear interpolation. Call `allocate` once outside the audio thread.
#[derive(Clone, Debug, Default)]
pub struct DelayLine {
    buf: Vec<f32>,
    pos: usize,
}

impl DelayLine {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            pos: 0,
        }
    }

    /// Allocates room for `max_samples` of delay. NOT for the audio thread.
    pub fn allocate(&mut self, max_samples: usize) {
        self.buf = vec![0.0; max_samples.max(4) + 4];
        self.pos = 0;
    }

    pub fn reset(&mut self) {
        self.buf.fill(0.0);
        self.pos = 0;
    }

    pub fn capacity(&self) -> usize {
        self.buf.len().saturating_sub(4)
    }

    #[inline]
    pub fn push(&mut self, x: f32) {
        if self.buf.is_empty() {
            return;
        }
        self.pos += 1;
        if self.pos == self.buf.len() {
            self.pos = 0;
        }
        self.buf[self.pos] = x;
    }

    /// Sample written `i` pushes ago (0 = newest).
    #[inline]
    pub fn tap(&self, i: usize) -> f32 {
        let n = self.buf.len();
        if n == 0 {
            return 0.0;
        }
        let idx = (self.pos + n - (i % n)) % n;
        self.buf[idx]
    }

    /// Fractional read, `delay` in samples (0 = newest). Clamped to [0, capacity].
    #[inline]
    pub fn read(&self, delay: f32) -> f32 {
        if self.buf.is_empty() {
            return 0.0;
        }
        let d = delay.clamp(0.0, self.capacity() as f32);
        let i = d.floor() as usize;
        let frac = d - i as f32;
        let a = self.tap(i);
        let b = self.tap(i + 1);
        a + (b - a) * frac
    }
}

/// Phase accumulator in [0, 1).
#[derive(Clone, Copy, Debug, Default)]
pub struct Phasor {
    phase: f32,
    inc: f32,
}

impl Phasor {
    pub fn set_freq(&mut self, sample_rate: f32, hz: f32) {
        self.inc = hz / sample_rate;
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.phase = phase.rem_euclid(1.0);
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn phase(&self) -> f32 {
        self.phase
    }

    /// Returns the current phase, then advances by one sample.
    #[inline]
    pub fn step(&mut self) -> f32 {
        let p = self.phase;
        self.phase += self.inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        p
    }
}

/// xorshift32 noise.
#[derive(Clone, Copy, Debug)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }

    /// Uniform in [-1, 1].
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;

    const SR: f32 = 48000.0;

    fn filter_amp(f: &mut impl FnMut(f32) -> f32, freq: f32) -> f32 {
        let x = sine(freq, SR, 4800, 1.0);
        let y: Vec<f32> = x.iter().map(|&v| f(v)).collect();
        tone_amplitude(&y[480..], freq, SR)
    }

    #[test]
    fn db_conversions() {
        assert!((db_to_gain(6.0) - 1.9953).abs() < 1e-3);
        assert!((gain_to_db(0.5) + 6.0206).abs() < 1e-3);
        assert!((mix(1.0, 3.0, 0.5) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn smoother_reaches_99_percent_in_set_time() {
        let mut s = Smoother::new(0.0);
        s.set_time(SR, 20.0);
        s.set_target(1.0);
        for _ in 0..30 {
            s.block(SUB);
        }
        assert!(s.value() >= 0.99 && s.value() <= 1.0, "{}", s.value());
    }

    #[test]
    fn smoother_block_returns_before_and_after() {
        let mut s = Smoother::new(2.0);
        s.set_time(SR, 0.0);
        s.set_target(5.0);
        assert_eq!(s.block(32), (2.0, 5.0));
    }

    #[test]
    fn glide_steps_linearly() {
        let mut g = Glide::default();
        g.set(0.0, 1.0, 4);
        assert_eq!(g.step(), 0.25);
        assert_eq!(g.step(), 0.5);
        assert_eq!(g.step(), 0.75);
        assert_eq!(g.step(), 1.0);
    }

    #[test]
    fn switch_fades_off_within_512_samples() {
        let mut s = Switch::new(true);
        s.set_sample_rate(SR);
        s.set(false);
        let (before, after) = s.block(32);
        assert_eq!(before, 1.0);
        assert!(after < 1.0);
        for _ in 0..15 {
            s.block(32);
        }
        assert!(s.is_off());
    }

    #[test]
    fn one_pole_lowpass_is_minus_3db_at_cutoff_and_complementary() {
        let mut lp = OnePole::default();
        lp.set_cutoff(SR, 1000.0);
        let a = filter_amp(&mut |v| lp.lowpass(v), 1000.0);
        assert!((a - 0.7071).abs() < 0.02, "{a}");
        lp.reset();
        assert!(filter_amp(&mut |v| lp.lowpass(v), 100.0) > 0.99);
        lp.reset();
        let (l, h) = lp.tick(0.3);
        assert!((l + h - 0.3).abs() < 1e-6);
    }

    #[test]
    fn biquad_lowpass_and_highpass() {
        let mut f = Biquad::default();
        f.set_lowpass(SR, 1000.0, std::f32::consts::FRAC_1_SQRT_2);
        assert!((filter_amp(&mut |v| f.tick(v), 1000.0) - 0.7071).abs() < 0.02);
        f.reset();
        assert!(filter_amp(&mut |v| f.tick(v), 10000.0) < 0.02);
        f.reset();
        assert!(filter_amp(&mut |v| f.tick(v), 100.0) > 0.99);
        let mut h = Biquad::default();
        h.set_highpass(SR, 1000.0, std::f32::consts::FRAC_1_SQRT_2);
        assert!((filter_amp(&mut |v| h.tick(v), 1000.0) - 0.7071).abs() < 0.02);
    }

    #[test]
    fn delay_line_reads_back_with_interpolation() {
        let mut d = DelayLine::new();
        d.allocate(16);
        for i in 0..10 {
            d.push(i as f32);
        }
        assert_eq!(d.read(0.0), 9.0);
        assert_eq!(d.read(3.0), 6.0);
        assert!((d.read(2.5) - 6.5).abs() < 1e-6);
        d.reset();
        assert_eq!(d.read(1.0), 0.0);
        let empty = DelayLine::new();
        assert_eq!(empty.read(3.0), 0.0);
    }

    #[test]
    fn dc_blocker_removes_dc() {
        let mut b = DcBlocker::default();
        b.set_sample_rate(SR);
        let mut y = 1.0;
        for _ in 0..48000 {
            y = b.tick(1.0);
        }
        assert!(y.abs() < 0.01, "{y}");
    }

    #[test]
    fn rng_is_bounded_and_deterministic() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let mut sum_abs = 0.0;
        for _ in 0..1000 {
            let x = a.next_f32();
            assert!(x >= -1.0 && x <= 1.0);
            assert_eq!(x, b.next_f32());
            sum_abs += x.abs();
        }
        assert!(sum_abs / 1000.0 > 0.3);
    }

    #[test]
    fn phasor_wraps() {
        let mut p = Phasor::default();
        p.set_freq(SR, 12000.0);
        assert_eq!(p.step(), 0.0);
        assert_eq!(p.step(), 0.25);
        assert_eq!(p.step(), 0.5);
        assert_eq!(p.step(), 0.75);
        assert_eq!(p.step(), 0.0);
    }
}
