//! Fuzz: coupling high-pass, gain, a bias-controlled gate and an asymmetric transistor-style
//! shaper, DC blocker, low-pass tone (1–10 kHz) and volume.
//! Runs inside the oversampler: `set_sample_rate` receives the OVERSAMPLED rate.

use crate::dsp::util::{db_to_gain, mix, Biquad, DcBlocker, Glide, OnePole, Smoother, Switch};
use std::f32::consts::FRAC_1_SQRT_2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FuzzSettings {
    pub on: bool,
    pub fuzz: f32,
    pub bias: f32,
    pub tone: f32,
    pub volume_db: f32,
}

impl Default for FuzzSettings {
    fn default() -> Self {
        Self {
            on: true,
            fuzz: 0.6,
            bias: 0.7,
            tone: 0.5,
            volume_db: 0.0,
        }
    }
}

#[inline]
fn shape(v: f32) -> f32 {
    if v >= 0.0 {
        v.tanh()
    } else {
        0.7 * (v / 0.7).tanh()
    }
}

#[derive(Clone, Copy, Default)]
struct Channel {
    hpf: OnePole,
    dc: DcBlocker,
    lp: Biquad,
}

impl Channel {
    fn clear(&mut self) {
        self.hpf.reset();
        self.dc.reset();
        self.lp.reset();
    }

    #[inline]
    fn tick(&mut self, x: f32, gain: f32, gate: f32, offset: f32) -> f32 {
        let pre = gain * self.hpf.highpass(x);
        let s = pre.signum() * (pre.abs() - gate).max(0.0);
        let y = shape(s + offset) - shape(offset);
        self.lp.tick(self.dc.tick(y)) * 0.5
    }
}

pub struct Fuzz {
    sr: f32,
    s: FuzzSettings,
    switch: Switch,
    fuzz: Smoother,
    bias: Smoother,
    tone: Smoother,
    volume: Smoother,
    mix_g: Glide,
    gain_g: Glide,
    vol_g: Glide,
    gate: f32,
    offset: f32,
    ch: [Channel; 2],
}

impl Fuzz {
    pub fn new() -> Self {
        let s = FuzzSettings::default();
        let mut f = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            fuzz: Smoother::new(s.fuzz),
            bias: Smoother::new(s.bias),
            tone: Smoother::new(s.tone),
            volume: Smoother::new(s.volume_db),
            mix_g: Glide::default(),
            gain_g: Glide::default(),
            vol_g: Glide::default(),
            gate: 0.0,
            offset: 0.0,
            ch: [Channel::default(); 2],
        };
        f.set_sample_rate(48000.0);
        f.reset();
        f
    }

    /// `sr` is the OVERSAMPLED rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [
            &mut self.fuzz,
            &mut self.bias,
            &mut self.tone,
            &mut self.volume,
        ] {
            sm.set_time(sr, 20.0);
        }
        for c in self.ch.iter_mut() {
            c.hpf.set_cutoff(sr, 80.0);
            c.dc.set_sample_rate(sr);
        }
        self.apply_block_values();
    }

    pub fn set(&mut self, s: FuzzSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.fuzz.set_target(s.fuzz);
        self.bias.set_target(s.bias);
        self.tone.set_target(s.tone);
        self.volume.set_target(s.volume_db);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.fuzz.reset(self.s.fuzz);
        self.bias.reset(self.s.bias);
        self.tone.reset(self.s.tone);
        self.volume.reset(self.s.volume_db);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
        self.gain_g.hold(db_to_gain(40.0 * self.fuzz.value()));
        self.vol_g.hold(db_to_gain(self.volume.value()));
    }

    fn apply_block_values(&mut self) {
        let b = self.bias.value();
        self.gate = (1.0 - b) * (1.0 - b) * 0.5;
        self.offset = 0.25 * (1.0 - b);
        let fc = 1000.0 * 10.0f32.powf(self.tone.value());
        let sr = self.sr;
        for c in self.ch.iter_mut() {
            c.lp.set_lowpass(sr, fc, FRAC_1_SQRT_2);
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
        let (f0, f1) = self.fuzz.block(frames);
        self.gain_g
            .set(db_to_gain(40.0 * f0), db_to_gain(40.0 * f1), frames);
        let (v0, v1) = self.volume.block(frames);
        self.vol_g.set(db_to_gain(v0), db_to_gain(v1), frames);
        self.bias.block(frames);
        self.tone.block(frames);
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let g = self.gain_g.step();
        let v = self.vol_g.step();
        if m == 0.0 {
            return (l, r);
        }
        let yl = self.ch[0].tick(l, g, self.gate, self.offset) * v;
        let yr = self.ch[1].tick(r, g, self.gate, self.offset) * v;
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: FuzzSettings) -> Fuzz {
        let mut f = Fuzz::new();
        f.set_sample_rate(SR);
        f.set(s);
        f.reset();
        f
    }

    fn run(f: &mut Fuzz, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            f.begin_block(chunk.len());
            for &v in chunk {
                out.push(f.process_frame(v, v).0);
            }
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut f = make(FuzzSettings {
            on: false,
            ..Default::default()
        });
        let x = noise(1000, 0.5, 13);
        assert_eq!(run(&mut f, &x), x);
    }

    #[test]
    fn low_bias_gates_small_signals() {
        let x = sine(200.0, SR, 9600, 0.005);
        let level = |bias: f32| {
            let mut f = make(FuzzSettings {
                fuzz: 0.0,
                bias,
                tone: 1.0,
                ..Default::default()
            });
            let y = run(&mut f, &x);
            rms(&y[4800..])
        };
        let starved = level(0.0);
        let full = level(1.0);
        assert!(full > 0.0005, "{full}");
        assert!(starved < full * 0.1, "starved {starved}, full {full}");
    }

    #[test]
    fn tone_controls_treble() {
        let x = sine(6000.0, SR, 52800, 0.01);
        let amp = |tone: f32| {
            let mut f = make(FuzzSettings {
                fuzz: 0.0,
                bias: 1.0,
                tone,
                ..Default::default()
            });
            let y = run(&mut f, &x);
            tone_amplitude(&y[4800..52800], 6000.0, SR)
        };
        assert!(db(amp(1.0)) > db(amp(0.0)) + 15.0);
    }

    #[test]
    fn more_fuzz_means_more_harmonics() {
        let ratio = |fuzz: f32| {
            let mut f = make(FuzzSettings {
                fuzz,
                bias: 1.0,
                tone: 1.0,
                ..Default::default()
            });
            let y = run(&mut f, &sine(200.0, SR, 52800, 0.1));
            let w = &y[4800..52800];
            tone_amplitude(w, 600.0, SR) / tone_amplitude(w, 200.0, SR)
        };
        let low = ratio(0.0);
        let high = ratio(1.0);
        assert!(low < 0.05, "{low}");
        assert!(high > 0.15, "{high}");
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 1.0, 14);
        for (fuzz, bias, tone, vol) in [
            (0.0, 0.0, 0.0, -24.0),
            (1.0, 1.0, 1.0, 12.0),
            (1.0, 0.0, 0.5, 12.0),
        ] {
            let mut f = make(FuzzSettings {
                on: true,
                fuzz,
                bias,
                tone,
                volume_db: vol,
            });
            let y = run(&mut f, &x);
            assert!(all_finite(&y));
            assert!(peak(&y) < 10.0, "{}", peak(&y));
        }
    }
}
