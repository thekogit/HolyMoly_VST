//! Drive (after the Suhr Riot): pre-emphasis high-pass, up to +50 dB of gain, a clipper,
//! a tilt tone control around 800 Hz and an output level.
//! Runs inside the oversampler: `set_sample_rate` receives the OVERSAMPLED rate.

use crate::dsp::util::{db_to_gain, mix, DcBlocker, Glide, OnePole, Smoother, Switch};
use crate::params::DriveVoice;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DriveSettings {
    pub on: bool,
    pub dist: f32,
    pub tone: f32,
    pub level_db: f32,
    pub voice: DriveVoice,
}

impl Default for DriveSettings {
    fn default() -> Self {
        Self {
            on: true,
            dist: 0.5,
            tone: 0.5,
            level_db: 0.0,
            voice: DriveVoice::Classic,
        }
    }
}

/// Bass cut before the clipper, per voice.
fn hpf_cutoff(voice: DriveVoice) -> f32 {
    match voice {
        DriveVoice::Tight => 600.0,
        DriveVoice::Classic => 200.0,
        DriveVoice::Loose => 40.0,
    }
}

/// Tight: hard symmetric. Classic: asymmetric soft. Loose: softest.
#[inline]
fn clip(voice: DriveVoice, x: f32) -> f32 {
    match voice {
        DriveVoice::Tight => x.clamp(-1.0, 1.0),
        DriveVoice::Classic => (x + 0.2).tanh() - 0.2f32.tanh(),
        DriveVoice::Loose => x / (1.0 + x * x).sqrt(),
    }
}

#[derive(Clone, Copy, Default)]
struct Channel {
    hpf: OnePole,
    dc: DcBlocker,
    tilt: OnePole,
}

impl Channel {
    fn clear(&mut self) {
        self.hpf.reset();
        self.dc.reset();
        self.tilt.reset();
    }

    #[inline]
    fn tick(&mut self, x: f32, gain: f32, voice: DriveVoice, lo: f32, hi: f32) -> f32 {
        let u = self.hpf.highpass(x);
        let c = self.dc.tick(clip(voice, gain * u) * 0.5);
        let (lp, hp) = self.tilt.tick(c);
        lp * lo + hp * hi
    }
}

pub struct Drive {
    sr: f32,
    s: DriveSettings,
    switch: Switch,
    dist: Smoother,
    tone: Smoother,
    level: Smoother,
    mix_g: Glide,
    gain_g: Glide,
    level_g: Glide,
    lo: f32,
    hi: f32,
    ch: [Channel; 2],
}

impl Drive {
    pub fn new() -> Self {
        let s = DriveSettings::default();
        let mut d = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            dist: Smoother::new(s.dist),
            tone: Smoother::new(s.tone),
            level: Smoother::new(s.level_db),
            mix_g: Glide::default(),
            gain_g: Glide::default(),
            level_g: Glide::default(),
            lo: 1.0,
            hi: 1.0,
            ch: [Channel::default(); 2],
        };
        d.set_sample_rate(48000.0);
        d.reset();
        d
    }

    /// `sr` is the OVERSAMPLED rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [&mut self.dist, &mut self.tone, &mut self.level] {
            sm.set_time(sr, 20.0);
        }
        let fc = hpf_cutoff(self.s.voice);
        for c in self.ch.iter_mut() {
            c.hpf.set_cutoff(sr, fc);
            c.dc.set_sample_rate(sr);
            c.tilt.set_cutoff(sr, 800.0);
        }
    }

    pub fn set(&mut self, s: DriveSettings) {
        let voice_changed = s.voice != self.s.voice;
        self.s = s;
        self.switch.set(s.on);
        self.dist.set_target(s.dist);
        self.tone.set_target(s.tone);
        self.level.set_target(s.level_db);
        if voice_changed {
            let (sr, fc) = (self.sr, hpf_cutoff(s.voice));
            for c in self.ch.iter_mut() {
                c.hpf.set_cutoff(sr, fc);
            }
        }
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.dist.reset(self.s.dist);
        self.tone.reset(self.s.tone);
        self.level.reset(self.s.level_db);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.update_tilt();
        self.mix_g.hold(self.switch.value());
        self.gain_g.hold(db_to_gain(50.0 * self.dist.value()));
        self.level_g.hold(db_to_gain(self.level.value()));
    }

    fn update_tilt(&mut self) {
        let t = self.tone.value() * 2.0 - 1.0;
        self.lo = db_to_gain(-9.0 * t);
        self.hi = db_to_gain(9.0 * t);
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
        let (d0, d1) = self.dist.block(frames);
        self.gain_g
            .set(db_to_gain(50.0 * d0), db_to_gain(50.0 * d1), frames);
        let (l0, l1) = self.level.block(frames);
        self.level_g.set(db_to_gain(l0), db_to_gain(l1), frames);
        self.tone.block(frames);
        self.update_tilt();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let g = self.gain_g.step();
        let lv = self.level_g.step();
        if m == 0.0 {
            return (l, r);
        }
        let v = self.s.voice;
        let yl = self.ch[0].tick(l, g, v, self.lo, self.hi) * lv;
        let yr = self.ch[1].tick(r, g, v, self.lo, self.hi) * lv;
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: DriveSettings) -> Drive {
        let mut d = Drive::new();
        d.set_sample_rate(SR);
        d.set(s);
        d.reset();
        d
    }

    fn run(d: &mut Drive, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            d.begin_block(chunk.len());
            for &v in chunk {
                out.push(d.process_frame(v, v).0);
            }
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut d = make(DriveSettings {
            on: false,
            ..Default::default()
        });
        let x = noise(1000, 0.5, 3);
        assert_eq!(run(&mut d, &x), x);
    }

    fn h3_ratio(dist: f32) -> f32 {
        let mut d = make(DriveSettings {
            dist,
            ..Default::default()
        });
        let y = run(&mut d, &sine(200.0, SR, 52800, 0.1));
        let w = &y[4800..52800];
        tone_amplitude(w, 600.0, SR) / tone_amplitude(w, 200.0, SR)
    }

    #[test]
    fn more_dist_means_more_harmonics() {
        let low = h3_ratio(0.0);
        let high = h3_ratio(1.0);
        assert!(low < 0.05, "dist 0: {low}");
        assert!(high > 0.2, "dist 1: {high}");
    }

    #[test]
    fn level_is_applied_in_db() {
        let x = sine(200.0, SR, 9600, 0.1);
        let mut a = make(DriveSettings {
            level_db: 0.0,
            ..Default::default()
        });
        let mut b = make(DriveSettings {
            level_db: 12.0,
            ..Default::default()
        });
        let ya = run(&mut a, &x);
        let yb = run(&mut b, &x);
        let ratio = rms(&yb[4800..]) / rms(&ya[4800..]);
        assert!((ratio - db_to_gain(12.0)).abs() < 0.05, "{ratio}");
    }

    #[test]
    fn tight_cuts_more_bass_than_loose() {
        let x = sine(80.0, SR, 52800, 0.001);
        let amp = |voice: DriveVoice| {
            let mut d = make(DriveSettings {
                dist: 0.0,
                voice,
                ..Default::default()
            });
            let y = run(&mut d, &x);
            tone_amplitude(&y[4800..52800], 80.0, SR)
        };
        let tight = amp(DriveVoice::Tight);
        let loose = amp(DriveVoice::Loose);
        assert!(tight < loose * 0.5, "tight {tight}, loose {loose}");
    }

    #[test]
    fn tone_tilts_treble() {
        let x = sine(5000.0, SR, 52800, 0.01);
        let amp = |tone: f32| {
            let mut d = make(DriveSettings {
                dist: 0.0,
                tone,
                ..Default::default()
            });
            let y = run(&mut d, &x);
            tone_amplitude(&y[4800..52800], 5000.0, SR)
        };
        assert!(db(amp(1.0)) > db(amp(0.0)) + 10.0);
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 1.0, 9);
        for voice in DriveVoice::ALL {
            for (dist, tone, level) in [(0.0, 0.0, -24.0), (1.0, 1.0, 12.0), (1.0, 0.0, 12.0)] {
                let mut d = make(DriveSettings {
                    on: true,
                    dist,
                    tone,
                    level_db: level,
                    voice,
                });
                let y = run(&mut d, &x);
                assert!(all_finite(&y));
                assert!(
                    peak(&y) < 20.0,
                    "{voice:?} {dist} {tone} {level}: {}",
                    peak(&y)
                );
            }
        }
    }
}
