//! Echo (after the Stereo Memory Man): stereo delay up to 3 s with a soft limiter in the
//! feedback loop, Digital / Tape / Multi / Reverse, tempo sync, ping-pong and trails.

use crate::dsp::util::{DelayLine, Glide, OnePole, Phasor, Rng, Smoother, Switch};
use crate::dsp::vibe::mix_gains;
use crate::params::{Division, EchoMode};
use std::f32::consts::{PI, TAU};

/// Reverse mode reads up to 2 x Time (3 s), plus modulation headroom.
const MAX_SECONDS: f32 = 6.2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EchoSettings {
    pub on: bool,
    pub time_ms: f32,
    pub sync: bool,
    pub division: Division,
    pub feedback: f32,
    pub mix: f32,
    pub tone_hz: f32,
    pub modulation: f32,
    pub mode: EchoMode,
    pub pingpong: bool,
    pub trails: bool,
    /// Host tempo in BPM; 0 or less means unknown (120 is used).
    pub tempo_bpm: f32,
}

impl Default for EchoSettings {
    fn default() -> Self {
        Self {
            on: true,
            time_ms: 400.0,
            sync: false,
            division: Division::Quarter,
            feedback: 0.35,
            mix: 0.35,
            tone_hz: 6000.0,
            modulation: 0.1,
            mode: EchoMode::Digital,
            pingpong: false,
            trails: true,
            tempo_bpm: 120.0,
        }
    }
}

impl EchoSettings {
    /// Delay time in ms after applying Sync (plan.md C1a).
    pub fn effective_time_ms(&self) -> f32 {
        if self.sync {
            let bpm = if self.tempo_bpm > 1.0 {
                self.tempo_bpm
            } else {
                120.0
            };
            (60000.0 / bpm * self.division.beats()).clamp(10.0, 3000.0)
        } else {
            self.time_ms.clamp(10.0, 3000.0)
        }
    }
}

struct Channel {
    line: DelayLine,
    tone: OnePole,
    rev_phase: f32,
}

impl Channel {
    fn new() -> Self {
        Self {
            line: DelayLine::new(),
            tone: OnePole::default(),
            rev_phase: 0.0,
        }
    }

    fn clear(&mut self) {
        self.line.reset();
        self.tone.reset();
        self.rev_phase = 0.0;
    }

    /// Reads the delayed signal BEFORE this frame's write. `d` = delay in samples.
    #[inline]
    fn read(&mut self, mode: EchoMode, d: f32) -> f32 {
        match mode {
            EchoMode::Multi => {
                let t1 = self.line.read(d / 3.0 - 1.0);
                let t2 = self.line.read(d * 2.0 / 3.0 - 1.0);
                let t3 = self.line.read(d - 1.0);
                0.6 * (0.5 * t1 + 0.75 * t2 + t3)
            }
            EchoMode::Reverse => {
                self.rev_phase += 1.0 / d.max(1.0);
                if self.rev_phase >= 1.0 {
                    self.rev_phase -= 1.0;
                }
                let p1 = self.rev_phase;
                let mut p2 = p1 + 0.5;
                if p2 >= 1.0 {
                    p2 -= 1.0;
                }
                let w = (PI * p1).sin();
                let w1 = w * w;
                self.line.read(2.0 * d * p1) * w1 + self.line.read(2.0 * d * p2) * (1.0 - w1)
            }
            EchoMode::Digital | EchoMode::Tape => self.line.read(d - 1.0),
        }
    }
}

pub struct Echo {
    sr: f32,
    s: EchoSettings,
    switch: Switch,
    time: Smoother,
    feedback: Smoother,
    mix_sm: Smoother,
    tone: Smoother,
    modulation_sm: Smoother,
    sw_g: Glide,
    time_g: Glide,
    fb: f32,
    dg: f32,
    wg: f32,
    modulation: f32,
    wow: Phasor,
    flutter: Phasor,
    noise_lp: OnePole,
    rng: Rng,
    ch: [Channel; 2],
}

impl Echo {
    /// Does not allocate the delay lines; call `prepare` before use.
    pub fn new() -> Self {
        let s = EchoSettings::default();
        let mut e = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            time: Smoother::new(s.effective_time_ms()),
            feedback: Smoother::new(s.feedback),
            mix_sm: Smoother::new(s.mix),
            tone: Smoother::new(s.tone_hz),
            modulation_sm: Smoother::new(s.modulation),
            sw_g: Glide::default(),
            time_g: Glide::default(),
            fb: s.feedback,
            dg: 1.0,
            wg: 0.7,
            modulation: s.modulation,
            wow: Phasor::default(),
            flutter: Phasor::default(),
            noise_lp: OnePole::default(),
            rng: Rng::new(0x1234_5678),
            ch: [Channel::new(), Channel::new()],
        };
        e.set_rates(48000.0);
        e.reset();
        e
    }

    fn set_rates(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        self.time.set_time(sr, 150.0);
        for sm in [
            &mut self.feedback,
            &mut self.mix_sm,
            &mut self.tone,
            &mut self.modulation_sm,
        ] {
            sm.set_time(sr, 20.0);
        }
        self.flutter.set_freq(sr, 6.0);
        self.noise_lp.set_cutoff(sr, 20.0);
    }

    /// Allocates 6.2 s of delay per channel. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        let cap = (MAX_SECONDS * sr) as usize + 8;
        for c in self.ch.iter_mut() {
            c.line.allocate(cap);
        }
        self.set_rates(sr);
        self.reset();
    }

    pub fn set(&mut self, s: EchoSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.time.set_target(s.effective_time_ms());
        self.feedback.set_target(s.feedback);
        self.mix_sm.set_target(s.mix);
        self.tone.set_target(s.tone_hz);
        self.modulation_sm.set_target(s.modulation);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.time.reset(self.s.effective_time_ms());
        self.feedback.reset(self.s.feedback);
        self.mix_sm.reset(self.s.mix);
        self.tone.reset(self.s.tone_hz);
        self.modulation_sm.reset(self.s.modulation);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.wow.reset();
        self.flutter.reset();
        self.noise_lp.reset();
        self.apply_block_values();
        self.time_g.hold(self.time.value() * self.sr / 1000.0);
        self.sw_g.hold(self.switch.value());
    }

    fn apply_block_values(&mut self) {
        self.fb = self.feedback.value();
        let (dg, wg) = mix_gains(self.mix_sm.value());
        self.dg = dg;
        self.wg = wg;
        self.modulation = self.modulation_sm.value();
        let tape = self.s.mode == EchoMode::Tape;
        let tone = if tape {
            self.tone.value() * 0.6
        } else {
            self.tone.value()
        };
        let sr = self.sr;
        for c in self.ch.iter_mut() {
            c.tone.set_cutoff(sr, tone);
        }
        self.wow.set_freq(sr, if tape { 0.5 } else { 0.8 });
    }

    /// False while trails are still allowed to ring.
    pub fn is_off(&self) -> bool {
        self.switch.is_off() && !self.s.trails
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on && !self.s.trails {
            for c in self.ch.iter_mut() {
                c.clear();
            }
        }
        let (m0, m1) = self.switch.block(frames);
        self.sw_g.set(m0, m1, frames);
        let (t0, t1) = self.time.block(frames);
        let k = self.sr / 1000.0;
        self.time_g.set(t0 * k, t1 * k, frames);
        self.feedback.block(frames);
        self.mix_sm.block(frames);
        self.tone.block(frames);
        self.modulation_sm.block(frames);
        self.apply_block_values();
    }

    /// Extra delay in samples from wow/flutter. Always >= 0.
    #[inline]
    fn mod_offset(&mut self) -> f32 {
        let wow = 0.5 + 0.5 * (TAU * self.wow.step()).sin();
        let flutter = 0.5 + 0.5 * (TAU * self.flutter.step()).sin();
        let n = self.noise_lp.lowpass(self.rng.next_f32()).abs();
        let amt = self.modulation * self.sr;
        if self.s.mode == EchoMode::Tape {
            amt * (0.004 * wow + 0.0004 * flutter + 0.0003 * n)
        } else {
            amt * 0.001 * wow
        }
    }

    #[inline]
    fn saturate(&self, v: f32) -> f32 {
        if self.s.mode == EchoMode::Tape {
            (1.5 * v).tanh() / 1.5
        } else {
            v.tanh()
        }
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.sw_g.step();
        let base = self.time_g.step();
        let off = self.mod_offset();
        if m == 0.0 && !self.s.trails {
            return (l, r);
        }
        let d = base + off;
        let mode = self.s.mode;
        let yl = self.ch[0].read(mode, d);
        let yr = self.ch[1].read(mode, d);
        let wl = self.ch[0].tone.lowpass(yl);
        let wr = self.ch[1].tone.lowpass(yr);
        let fl = self.saturate(self.fb * wl);
        let fr = self.saturate(self.fb * wr);
        let (il, ir) = (l * m, r * m);
        let (inl, inr) = if self.s.pingpong {
            ((il + ir) * 0.5 + fr, fl)
        } else {
            (il + fl, ir + fr)
        };
        self.ch[0].line.push(inl);
        self.ch[1].line.push(inr);
        let dry = 1.0 + (self.dg - 1.0) * m;
        let wet = if self.s.trails { self.wg } else { self.wg * m };
        (l * dry + wl * wet, r * dry + wr * wet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn base() -> EchoSettings {
        EchoSettings {
            on: true,
            time_ms: 400.0,
            sync: false,
            division: Division::Quarter,
            feedback: 0.0,
            mix: 1.0,
            tone_hz: 16000.0,
            modulation: 0.0,
            mode: EchoMode::Digital,
            pingpong: false,
            trails: false,
            tempo_bpm: 120.0,
        }
    }

    fn make(s: EchoSettings) -> Echo {
        let mut e = Echo::new();
        e.prepare(SR);
        e.set(s);
        e.reset();
        e
    }

    fn run(e: &mut Echo, l: &[f32], r: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut out_l = Vec::with_capacity(l.len());
        let mut out_r = Vec::with_capacity(r.len());
        let mut i = 0;
        while i < l.len() {
            let n = SUB.min(l.len() - i);
            e.begin_block(n);
            for k in i..i + n {
                let (a, b) = e.process_frame(l[k], r[k]);
                out_l.push(a);
                out_r.push(b);
            }
            i += n;
        }
        (out_l, out_r)
    }

    fn burst(len: usize, on: usize, freq: f32, amp: f32) -> Vec<f32> {
        let mut x = sine(freq, SR, len, amp);
        for v in x.iter_mut().skip(on) {
            *v = 0.0;
        }
        x
    }

    fn run_switch_off(s: EchoSettings, at: usize, x: &[f32]) -> Vec<f32> {
        let mut e = make(s);
        let mut out = Vec::with_capacity(x.len());
        let mut done = 0;
        for chunk in x.chunks(SUB) {
            if done >= at {
                e.set(EchoSettings { on: false, ..s });
            }
            e.begin_block(chunk.len());
            for &v in chunk {
                out.push(e.process_frame(v, v).0);
            }
            done += chunk.len();
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut e = make(EchoSettings {
            on: false,
            trails: false,
            ..base()
        });
        let x = noise(4800, 0.5, 51);
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(l, x);
    }

    #[test]
    fn first_repeat_arrives_after_time() {
        let mut e = make(base());
        let mut x = vec![0.0f32; 24000];
        x[0] = 1.0;
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(argmax_abs(&l), 19200);
    }

    #[test]
    fn sync_uses_host_tempo() {
        let mut e = make(EchoSettings {
            sync: true,
            tempo_bpm: 120.0,
            division: Division::Quarter,
            ..base()
        });
        let mut x = vec![0.0f32; 30000];
        x[0] = 1.0;
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(argmax_abs(&l), 24000);
    }

    #[test]
    fn feedback_sets_repeat_level() {
        let mut e = make(EchoSettings {
            time_ms: 200.0,
            feedback: 0.5,
            ..base()
        });
        let x = burst(21000, 960, 500.0, 0.1);
        let (l, _) = run(&mut e, &x, &x);
        let first = rms(&l[9600..10560]);
        let second = rms(&l[19200..20160]);
        let ratio = second / first;
        assert!(first > 0.05, "{first}");
        assert!(ratio > 0.4 && ratio < 0.6, "{ratio}");
    }

    #[test]
    fn feedback_above_100_percent_self_oscillates_safely() {
        let mut e = make(EchoSettings {
            time_ms: 100.0,
            feedback: 1.1,
            tone_hz: 6000.0,
            ..base()
        });
        let mut x = noise(144000, 0.5, 52);
        for v in x.iter_mut().skip(4800) {
            *v = 0.0;
        }
        let (l, r) = run(&mut e, &x, &x);
        assert!(all_finite(&l) && all_finite(&r));
        assert!(peak(&l) < 3.0, "{}", peak(&l));
        assert!(rms(&l[120000..]) > 0.01, "{}", rms(&l[120000..]));
    }

    #[test]
    fn pingpong_alternates_sides() {
        let mut e = make(EchoSettings {
            time_ms: 100.0,
            feedback: 0.5,
            pingpong: true,
            ..base()
        });
        let mut x = vec![0.0f32; 12000];
        x[0] = 1.0;
        let (l, r) = run(&mut e, &x, &x);
        assert!(peak(&l[4790..4810]) > 0.3);
        assert!(peak(&r[4790..4810]) < 0.01);
        assert!(peak(&r[9590..9610]) > 0.1);
        assert!(peak(&l[9590..9610]) < 0.01);
    }

    #[test]
    fn trails_ring_out_after_bypass() {
        let s = EchoSettings {
            time_ms: 100.0,
            feedback: 0.7,
            mix: 0.5,
            ..base()
        };
        let x = burst(48000, 960, 500.0, 0.3);
        let with = run_switch_off(EchoSettings { trails: true, ..s }, 2400, &x);
        let without = run_switch_off(EchoSettings { trails: false, ..s }, 2400, &x);
        assert!(rms(&with[24000..]) > 1e-4, "{}", rms(&with[24000..]));
        assert!(without[24000..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn multi_mode_has_three_taps() {
        let mut e = make(EchoSettings {
            time_ms: 300.0,
            mode: EchoMode::Multi,
            ..base()
        });
        let mut x = vec![0.0f32; 16000];
        x[0] = 1.0;
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(4000 + argmax_abs(&l[4000..5600]), 4800);
        assert_eq!(9000 + argmax_abs(&l[9000..10200]), 9600);
        assert_eq!(14000 + argmax_abs(&l[14000..15000]), 14400);
    }

    #[test]
    fn reverse_mode_produces_sound() {
        let mut e = make(EchoSettings {
            time_ms: 100.0,
            mode: EchoMode::Reverse,
            ..base()
        });
        let x = burst(24000, 2400, 500.0, 0.3);
        let (l, r) = run(&mut e, &x, &x);
        assert!(all_finite(&l) && all_finite(&r));
        assert!(rms(&l) > 0.01, "{}", rms(&l));
    }

    #[test]
    fn extremes_stay_finite_and_bounded() {
        let x = noise(9600, 0.5, 53);
        for mode in EchoMode::ALL {
            for (time, tone, pingpong) in [
                (10.0, 1000.0, false),
                (10.0, 16000.0, true),
                (3000.0, 16000.0, true),
            ] {
                let mut e = make(EchoSettings {
                    time_ms: time,
                    feedback: 1.1,
                    modulation: 1.0,
                    tone_hz: tone,
                    mode,
                    pingpong,
                    trails: true,
                    mix: 0.5,
                    ..base()
                });
                let (l, r) = run(&mut e, &x, &x);
                assert!(all_finite(&l) && all_finite(&r), "{mode:?}");
                assert!(
                    peak(&l) < 4.0 && peak(&r) < 4.0,
                    "{mode:?} {time}: {}",
                    peak(&l)
                );
            }
        }
    }
}
