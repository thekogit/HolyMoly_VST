//! Ring (after the DOD Gonkulator): Self = signal x clipped copy of itself, Osc = signal x sine.
//! Runs inside the oversampler: `set_sample_rate` receives the OVERSAMPLED rate.

use crate::dsp::util::{db_to_gain, mix, DcBlocker, Glide, Phasor, Smoother, Switch};
use crate::params::RingMode;
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingSettings {
    pub on: bool,
    pub gain_db: f32,
    pub dist: f32,
    pub modulation: f32,
    pub output_db: f32,
    pub mode: RingMode,
    pub freq_hz: f32,
}

impl Default for RingSettings {
    fn default() -> Self {
        Self {
            on: false,
            gain_db: 6.0,
            dist: 0.4,
            modulation: 1.0,
            output_db: 0.0,
            mode: RingMode::SelfMod,
            freq_hz: 440.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Channel {
    dc: DcBlocker,
}

impl Channel {
    #[inline]
    fn tick(
        &mut self,
        x: f32,
        gain: f32,
        osc: f32,
        mode: RingMode,
        modulation: f32,
        d: f32,
    ) -> f32 {
        let a = gain * x;
        let carrier = match mode {
            RingMode::Osc => osc,
            RingMode::SelfMod => (3.0 * a).tanh(),
        };
        let w = mix(a.tanh(), a * carrier, modulation);
        self.dc.tick((d * w).tanh() / d.sqrt())
    }
}

pub struct Ring {
    sr: f32,
    s: RingSettings,
    switch: Switch,
    gain: Smoother,
    dist: Smoother,
    modulation: Smoother,
    output: Smoother,
    freq: Smoother,
    mix_g: Glide,
    gain_g: Glide,
    out_g: Glide,
    d: f32,
    modu: f32,
    osc: Phasor,
    ch: [Channel; 2],
}

impl Ring {
    pub fn new() -> Self {
        let s = RingSettings::default();
        let mut r = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            gain: Smoother::new(s.gain_db),
            dist: Smoother::new(s.dist),
            modulation: Smoother::new(s.modulation),
            output: Smoother::new(s.output_db),
            freq: Smoother::new(s.freq_hz),
            mix_g: Glide::default(),
            gain_g: Glide::default(),
            out_g: Glide::default(),
            d: 1.0,
            modu: 1.0,
            osc: Phasor::default(),
            ch: [Channel::default(); 2],
        };
        r.set_sample_rate(48000.0);
        r.reset();
        r
    }

    /// `sr` is the OVERSAMPLED rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [
            &mut self.gain,
            &mut self.dist,
            &mut self.modulation,
            &mut self.output,
            &mut self.freq,
        ] {
            sm.set_time(sr, 20.0);
        }
        for c in self.ch.iter_mut() {
            c.dc.set_sample_rate(sr);
        }
        self.apply_block_values();
    }

    pub fn set(&mut self, s: RingSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.gain.set_target(s.gain_db);
        self.dist.set_target(s.dist);
        self.modulation.set_target(s.modulation);
        self.output.set_target(s.output_db);
        self.freq.set_target(s.freq_hz);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.gain.reset(self.s.gain_db);
        self.dist.reset(self.s.dist);
        self.modulation.reset(self.s.modulation);
        self.output.reset(self.s.output_db);
        self.freq.reset(self.s.freq_hz);
        self.clear_state();
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
        self.gain_g.hold(db_to_gain(self.gain.value()));
        self.out_g.hold(db_to_gain(self.output.value()));
    }

    fn clear_state(&mut self) {
        for c in self.ch.iter_mut() {
            c.dc.reset();
        }
        self.osc.reset();
    }

    fn apply_block_values(&mut self) {
        self.d = 1.0 + 9.0 * self.dist.value();
        self.modu = self.modulation.value();
        self.osc.set_freq(self.sr, self.freq.value());
    }

    pub fn is_off(&self) -> bool {
        self.switch.is_off()
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on {
            self.clear_state();
        }
        let (m0, m1) = self.switch.block(frames);
        self.mix_g.set(m0, m1, frames);
        let (g0, g1) = self.gain.block(frames);
        self.gain_g.set(db_to_gain(g0), db_to_gain(g1), frames);
        let (o0, o1) = self.output.block(frames);
        self.out_g.set(db_to_gain(o0), db_to_gain(o1), frames);
        self.dist.block(frames);
        self.modulation.block(frames);
        self.freq.block(frames);
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let g = self.gain_g.step();
        let o = self.out_g.step();
        let ph = self.osc.step();
        if m == 0.0 {
            return (l, r);
        }
        let osc = (TAU * ph).sin();
        let mode = self.s.mode;
        let yl = self.ch[0].tick(l, g, osc, mode, self.modu, self.d) * o;
        let yr = self.ch[1].tick(r, g, osc, mode, self.modu, self.d) * o;
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: RingSettings) -> Ring {
        let mut r = Ring::new();
        r.set_sample_rate(SR);
        r.set(s);
        r.reset();
        r
    }

    fn run(r: &mut Ring, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            r.begin_block(chunk.len());
            for &v in chunk {
                out.push(r.process_frame(v, v).0);
            }
        }
        out
    }

    fn clean(mode: RingMode, modulation: f32) -> RingSettings {
        RingSettings {
            on: true,
            gain_db: 0.0,
            dist: 0.0,
            modulation,
            output_db: 0.0,
            mode,
            freq_hz: 440.0,
        }
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut r = make(RingSettings {
            on: false,
            ..Default::default()
        });
        let x = noise(1000, 0.5, 21);
        assert_eq!(run(&mut r, &x), x);
    }

    #[test]
    fn osc_mode_makes_sum_and_difference_tones() {
        let mut r = make(clean(RingMode::Osc, 1.0));
        let y = run(&mut r, &sine(1000.0, SR, 52800, 0.3));
        let w = &y[4800..52800];
        assert!(tone_amplitude(w, 560.0, SR) > 0.08);
        assert!(tone_amplitude(w, 1440.0, SR) > 0.08);
        assert!(tone_amplitude(w, 1000.0, SR) < 0.01);
    }

    #[test]
    fn mod_zero_keeps_the_dry_note() {
        let mut r = make(clean(RingMode::Osc, 0.0));
        let y = run(&mut r, &sine(1000.0, SR, 52800, 0.3));
        let w = &y[4800..52800];
        assert!(tone_amplitude(w, 1000.0, SR) > 0.2);
        assert!(tone_amplitude(w, 560.0, SR) < 0.01);
    }

    #[test]
    fn self_mode_is_loud_and_dc_free() {
        let mut r = make(RingSettings {
            on: true,
            mode: RingMode::SelfMod,
            ..Default::default()
        });
        let y = run(&mut r, &sine(200.0, SR, 48000, 0.3));
        let tail = &y[24000..];
        let mean = tail.iter().sum::<f32>() / tail.len() as f32;
        assert!(rms(tail) > 0.01, "{}", rms(tail));
        assert!(mean.abs() < 0.01, "{mean}");
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 1.0, 22);
        for mode in RingMode::ALL {
            for (gain, dist, out) in [(0.0, 0.0, -24.0), (30.0, 1.0, 12.0)] {
                let mut r = make(RingSettings {
                    on: true,
                    gain_db: gain,
                    dist,
                    modulation: 1.0,
                    output_db: out,
                    mode,
                    freq_hz: 2000.0,
                });
                let y = run(&mut r, &x);
                assert!(all_finite(&y));
                assert!(peak(&y) < 10.0, "{}", peak(&y));
            }
        }
    }
}
