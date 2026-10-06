//! Prism (after the Rainbow Machine): dual-head delay-line pitch shifter with two voices,
//! "halo" feedback through the shifter, and a tone low-pass on the wet path.

use crate::dsp::util::{mix, Biquad, DelayLine, Glide, Phasor, Smoother, Switch};
use std::f32::consts::{FRAC_1_SQRT_2, PI, TAU};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrismSettings {
    pub on: bool,
    pub pitch_st: f32,
    pub primary: f32,
    pub secondary: f32,
    pub tracking_ms: f32,
    pub depth: f32,
    pub halo: f32,
    pub tone_hz: f32,
}

impl Default for PrismSettings {
    fn default() -> Self {
        Self {
            on: false,
            pitch_st: 12.0,
            primary: 0.5,
            secondary: 0.0,
            tracking_ms: 40.0,
            depth: 0.2,
            halo: 0.0,
            tone_hz: 8000.0,
        }
    }
}

/// Per-sub-block constants shared by both channels.
#[derive(Clone, Copy, Debug, Default)]
struct Block {
    r1: f32,
    r2: f32,
    grain: f32,
    extra: f32,
    primary: f32,
    secondary: f32,
    halo: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Head {
    phase: f32,
}

impl Head {
    #[inline]
    fn read(&mut self, line: &DelayLine, ratio: f32, grain: f32, extra: f32) -> f32 {
        self.phase = (self.phase + (ratio - 1.0) / grain).rem_euclid(1.0);
        let p1 = self.phase;
        let mut p2 = p1 + 0.5;
        if p2 >= 1.0 {
            p2 -= 1.0;
        }
        let w = (PI * p1).sin();
        let w1 = w * w;
        let d1 = 2.0 + extra + grain * (1.0 - p1);
        let d2 = 2.0 + extra + grain * (1.0 - p2);
        line.read(d1) * w1 + line.read(d2) * (1.0 - w1)
    }
}

struct Channel {
    line: DelayLine,
    v1: Head,
    v2: Head,
    t1: Biquad,
    t2: Biquad,
    last: f32,
    lfo: Phasor,
}

impl Channel {
    fn new() -> Self {
        Self {
            line: DelayLine::new(),
            v1: Head::default(),
            v2: Head::default(),
            t1: Biquad::default(),
            t2: Biquad::default(),
            last: 0.0,
            lfo: Phasor::default(),
        }
    }

    fn clear(&mut self) {
        self.line.reset();
        self.v1 = Head::default();
        self.v2 = Head::default();
        self.t1.reset();
        self.t2.reset();
        self.last = 0.0;
    }

    #[inline]
    fn tick(&mut self, x: f32, k: &Block) -> f32 {
        let fb = (k.halo * self.last).tanh();
        self.line.push(x + fb);
        let lfo = 0.5 + 0.5 * (TAU * self.lfo.step()).sin();
        let extra = k.extra * lfo;
        let a = self.t1.tick(self.v1.read(&self.line, k.r1, k.grain, extra));
        let b = self.t2.tick(self.v2.read(&self.line, k.r2, k.grain, extra));
        self.last = a + k.secondary * b;
        x * (1.0 - k.primary) + a * k.primary + b * k.secondary
    }
}

pub struct Prism {
    sr: f32,
    s: PrismSettings,
    switch: Switch,
    pitch: Smoother,
    primary: Smoother,
    secondary: Smoother,
    tracking: Smoother,
    depth: Smoother,
    halo: Smoother,
    tone: Smoother,
    mix_g: Glide,
    k: Block,
    ch: [Channel; 2],
}

impl Prism {
    /// Does not allocate the delay lines; call `prepare` before use.
    pub fn new() -> Self {
        let s = PrismSettings::default();
        Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            pitch: Smoother::new(s.pitch_st),
            primary: Smoother::new(s.primary),
            secondary: Smoother::new(s.secondary),
            tracking: Smoother::new(s.tracking_ms),
            depth: Smoother::new(s.depth),
            halo: Smoother::new(s.halo),
            tone: Smoother::new(s.tone_hz),
            mix_g: Glide::default(),
            k: Block::default(),
            ch: [Channel::new(), Channel::new()],
        }
    }

    /// Allocates 0.25 s of delay per channel. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        self.sr = sr;
        let cap = (0.25 * sr) as usize + 8;
        for c in self.ch.iter_mut() {
            c.line.allocate(cap);
            c.lfo.set_freq(sr, 0.7);
        }
        self.switch.set_sample_rate(sr);
        for sm in [
            &mut self.pitch,
            &mut self.primary,
            &mut self.secondary,
            &mut self.tracking,
            &mut self.depth,
            &mut self.halo,
            &mut self.tone,
        ] {
            sm.set_time(sr, 20.0);
        }
        self.reset();
    }

    pub fn set(&mut self, s: PrismSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.pitch.set_target(s.pitch_st);
        self.primary.set_target(s.primary);
        self.secondary.set_target(s.secondary);
        self.tracking.set_target(s.tracking_ms);
        self.depth.set_target(s.depth);
        self.halo.set_target(s.halo);
        self.tone.set_target(s.tone_hz);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.pitch.reset(self.s.pitch_st);
        self.primary.reset(self.s.primary);
        self.secondary.reset(self.s.secondary);
        self.tracking.reset(self.s.tracking_ms);
        self.depth.reset(self.s.depth);
        self.halo.reset(self.s.halo);
        self.tone.reset(self.s.tone_hz);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.ch[0].lfo.set_phase(0.0);
        self.ch[1].lfo.set_phase(0.25);
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
    }

    fn apply_block_values(&mut self) {
        let sr = self.sr;
        self.k.r1 = 2.0f32.powf(self.pitch.value() / 12.0);
        self.k.r2 = self.k.r1 * 2.0;
        self.k.grain = (self.tracking.value() * 0.001 * sr).max(16.0);
        self.k.extra = self.depth.value() * 0.004 * sr;
        self.k.primary = self.primary.value();
        self.k.secondary = self.secondary.value();
        self.k.halo = self.halo.value();
        let tone = self.tone.value();
        for c in self.ch.iter_mut() {
            c.t1.set_lowpass(sr, tone, FRAC_1_SQRT_2);
            c.t2.set_lowpass(sr, tone, FRAC_1_SQRT_2);
        }
    }

    pub fn is_off(&self) -> bool {
        self.switch.is_off()
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on {
            for c in self.ch.iter_mut() {
                c.clear();
            }
        }
        let (m0, m1) = self.switch.block(frames);
        self.mix_g.set(m0, m1, frames);
        for sm in [
            &mut self.pitch,
            &mut self.primary,
            &mut self.secondary,
            &mut self.tracking,
            &mut self.depth,
            &mut self.halo,
            &mut self.tone,
        ] {
            sm.block(frames);
        }
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        if m == 0.0 {
            return (l, r);
        }
        let k = self.k;
        let yl = self.ch[0].tick(l, &k);
        let yr = self.ch[1].tick(r, &k);
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: PrismSettings) -> Prism {
        let mut p = Prism::new();
        p.prepare(SR);
        p.set(s);
        p.reset();
        p
    }

    fn run(p: &mut Prism, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            p.begin_block(chunk.len());
            for &v in chunk {
                out.push(p.process_frame(v, v).0);
            }
        }
        out
    }

    /// Amplitude of everything within +-60 Hz of `center` (window must be exactly 1 s).
    fn band(x: &[f32], center: f32) -> f32 {
        let mut e = 0.0f32;
        let mut f = center - 60.0;
        while f <= center + 60.0 {
            let a = tone_amplitude(x, f, SR);
            e += a * a;
            f += 1.0;
        }
        e.sqrt()
    }

    fn clean(pitch_st: f32, primary: f32, secondary: f32) -> PrismSettings {
        PrismSettings {
            on: true,
            pitch_st,
            primary,
            secondary,
            tracking_ms: 40.0,
            depth: 0.0,
            halo: 0.0,
            tone_hz: 16000.0,
        }
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut p = make(PrismSettings {
            on: false,
            ..Default::default()
        });
        let x = noise(1000, 0.5, 41);
        assert_eq!(run(&mut p, &x), x);
    }

    #[test]
    fn plus_12_moves_energy_an_octave_up() {
        let mut p = make(clean(12.0, 1.0, 0.0));
        let y = run(&mut p, &sine(440.0, SR, 57600, 0.5));
        let w = &y[9600..57600];
        let up = band(w, 880.0);
        let orig = band(w, 440.0);
        assert!(up > 0.2, "{up}");
        assert!(up > 4.0 * orig, "up {up}, orig {orig}");
    }

    #[test]
    fn minus_12_moves_energy_an_octave_down() {
        let mut p = make(clean(-12.0, 1.0, 0.0));
        let y = run(&mut p, &sine(440.0, SR, 57600, 0.5));
        let w = &y[9600..57600];
        let down = band(w, 220.0);
        let orig = band(w, 440.0);
        assert!(down > 0.2, "{down}");
        assert!(down > 4.0 * orig, "down {down}, orig {orig}");
    }

    #[test]
    fn secondary_adds_an_octave_above_primary() {
        let mut p = make(clean(0.0, 1.0, 1.0));
        let y = run(&mut p, &sine(440.0, SR, 57600, 0.5));
        let w = &y[9600..57600];
        assert!(band(w, 880.0) > 0.15, "{}", band(w, 880.0));
        assert!(band(w, 440.0) > 0.15, "{}", band(w, 440.0));
    }

    #[test]
    fn halo_extremes_stay_finite_and_bounded() {
        let x = noise(19200, 0.5, 42);
        for tracking in [10.0, 120.0] {
            let mut p = make(PrismSettings {
                on: true,
                pitch_st: 12.0,
                primary: 1.0,
                secondary: 1.0,
                tracking_ms: tracking,
                depth: 1.0,
                halo: 0.95,
                tone_hz: 16000.0,
            });
            let y = run(&mut p, &x);
            assert!(all_finite(&y));
            assert!(peak(&y) < 5.0, "{}", peak(&y));
        }
    }
}
