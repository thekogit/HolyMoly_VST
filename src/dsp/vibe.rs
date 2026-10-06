//! Vibe: four first-order all-pass stages swept by a lamp-like LFO, with feedback.

use crate::dsp::util::{mix, Glide, Phasor, Smoother, Switch};
use crate::params::VibeMode;
use std::f32::consts::{PI, TAU};

const BASES: [f32; 4] = [180.0, 520.0, 1400.0, 3600.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VibeSettings {
    pub on: bool,
    pub rate_hz: f32,
    pub depth: f32,
    pub feedback: f32,
    pub mix: f32,
    pub mode: VibeMode,
}

impl Default for VibeSettings {
    fn default() -> Self {
        Self {
            on: true,
            rate_hz: 1.5,
            depth: 0.6,
            feedback: 0.3,
            mix: 0.5,
            mode: VibeMode::Chorus,
        }
    }
}

/// Pedal-style mix law: 0 = all dry, 0.5 = both full, 1 = all wet. Returns (dry gain, wet gain).
pub fn mix_gains(amount: f32) -> (f32, f32) {
    ((2.0 * (1.0 - amount)).min(1.0), (2.0 * amount).min(1.0))
}

/// LFO phase -> sweep factor (lamp-like: dwells at the low end).
#[inline]
fn sweep(phase: f32, depth: f32) -> f32 {
    let s = 0.5 + 0.5 * (TAU * phase).sin();
    2.0f32.powf(4.0 * depth * (s * s - 0.5))
}

#[derive(Clone, Copy, Default)]
struct Channel {
    x1: [f32; 4],
    y1: [f32; 4],
    last: f32,
    lfo: Phasor,
}

impl Channel {
    fn clear(&mut self) {
        self.x1 = [0.0; 4];
        self.y1 = [0.0; 4];
        self.last = 0.0;
    }

    #[inline]
    fn tick(&mut self, x: f32, factor: f32, fb: f32, sr: f32) -> f32 {
        let mut u = x + fb * self.last;
        for k in 0..4 {
            let f = (BASES[k] * factor).clamp(20.0, 0.45 * sr);
            let t = (PI * f / sr).tan();
            let a = (t - 1.0) / (t + 1.0);
            let y = a * u + self.x1[k] - a * self.y1[k];
            self.x1[k] = u;
            self.y1[k] = y;
            u = y;
        }
        self.last = u;
        u
    }
}

pub struct Vibe {
    sr: f32,
    s: VibeSettings,
    switch: Switch,
    depth: Smoother,
    feedback: Smoother,
    mix_sm: Smoother,
    mix_g: Glide,
    depth_v: f32,
    fb_v: f32,
    dg: f32,
    wg: f32,
    ch: [Channel; 2],
}

impl Vibe {
    pub fn new() -> Self {
        let s = VibeSettings::default();
        let mut v = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            depth: Smoother::new(s.depth),
            feedback: Smoother::new(s.feedback),
            mix_sm: Smoother::new(s.mix),
            mix_g: Glide::default(),
            depth_v: s.depth,
            fb_v: s.feedback,
            dg: 1.0,
            wg: 1.0,
            ch: [Channel::default(); 2],
        };
        v.set_sample_rate(48000.0);
        v.reset();
        v
    }

    /// Host rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [&mut self.depth, &mut self.feedback, &mut self.mix_sm] {
            sm.set_time(sr, 20.0);
        }
        self.apply_block_values();
    }

    pub fn set(&mut self, s: VibeSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.depth.set_target(s.depth);
        self.feedback.set_target(s.feedback);
        self.mix_sm.set_target(s.mix);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.depth.reset(self.s.depth);
        self.feedback.reset(self.s.feedback);
        self.mix_sm.reset(self.s.mix);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.ch[0].lfo.set_phase(0.0);
        self.ch[1].lfo.set_phase(0.25);
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
    }

    fn apply_block_values(&mut self) {
        self.depth_v = self.depth.value();
        self.fb_v = self.feedback.value();
        let (dg, wg) = mix_gains(self.mix_sm.value());
        self.dg = dg;
        self.wg = wg;
        let (sr, rate) = (self.sr, self.s.rate_hz);
        for c in self.ch.iter_mut() {
            c.lfo.set_freq(sr, rate);
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
        self.depth.block(frames);
        self.feedback.block(frames);
        self.mix_sm.block(frames);
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let pl = self.ch[0].lfo.step();
        let pr = self.ch[1].lfo.step();
        if m == 0.0 {
            return (l, r);
        }
        let sr = self.sr;
        let wl = self.ch[0].tick(l, sweep(pl, self.depth_v), self.fb_v, sr);
        let wr = self.ch[1].tick(r, sweep(pr, self.depth_v), self.fb_v, sr);
        let (out_l, out_r) = match self.s.mode {
            VibeMode::Vibrato => (wl, wr),
            VibeMode::Chorus => {
                let norm = self.dg + self.wg;
                (
                    (l * self.dg + wl * self.wg) / norm,
                    (r * self.dg + wr * self.wg) / norm,
                )
            }
        };
        (mix(l, out_l, m), mix(r, out_r, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: VibeSettings) -> Vibe {
        let mut v = Vibe::new();
        v.set_sample_rate(SR);
        v.set(s);
        v.reset();
        v
    }

    fn run(v: &mut Vibe, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            v.begin_block(chunk.len());
            for &s in chunk {
                out.push(v.process_frame(s, s).0);
            }
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut v = make(VibeSettings {
            on: false,
            ..Default::default()
        });
        let x = noise(1000, 0.5, 31);
        assert_eq!(run(&mut v, &x), x);
    }

    #[test]
    fn chorus_with_mix_zero_is_dry() {
        let mut v = make(VibeSettings {
            mix: 0.0,
            ..Default::default()
        });
        let x = noise(4800, 0.5, 32);
        assert_eq!(run(&mut v, &x), x);
    }

    #[test]
    fn vibrato_without_depth_is_an_allpass() {
        let mut v = make(VibeSettings {
            mode: VibeMode::Vibrato,
            depth: 0.0,
            feedback: 0.0,
            ..Default::default()
        });
        let x = sine(1000.0, SR, 9600, 0.5);
        let y = run(&mut v, &x);
        let ratio = rms(&y[4800..]) / rms(&x[4800..]);
        assert!((ratio - 1.0).abs() < 0.01, "{ratio}");
    }

    #[test]
    fn lfo_sweeps_the_notches() {
        let mut v = make(VibeSettings {
            on: true,
            rate_hz: 2.0,
            depth: 1.0,
            feedback: 0.0,
            mix: 0.5,
            mode: VibeMode::Chorus,
        });
        let x = sine(1000.0, SR, 48000, 0.5);
        let y = run(&mut v, &x);
        let windows: Vec<f32> = y[4800..].chunks(480).map(rms).collect();
        let max = windows.iter().cloned().fold(0.0f32, f32::max);
        let min = windows.iter().cloned().fold(f32::MAX, f32::min);
        assert!(max > 2.0 * min, "max {max}, min {min}");
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 0.5, 33);
        for mode in VibeMode::ALL {
            let mut v = make(VibeSettings {
                on: true,
                rate_hz: 10.0,
                depth: 1.0,
                feedback: 0.9,
                mix: 1.0,
                mode,
            });
            let y = run(&mut v, &x);
            assert!(all_finite(&y));
            assert!(peak(&y) < 10.0, "{}", peak(&y));
        }
    }
}
