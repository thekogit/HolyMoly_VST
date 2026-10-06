//! The fixed signal chain. Plain Rust: no nih-plug. Built from `ChainSettings` once per host block.

use crate::dsp::drive::{Drive, DriveSettings};
use crate::dsp::echo::{Echo, EchoSettings};
use crate::dsp::fuzz::{Fuzz, FuzzSettings};
use crate::dsp::looper::{Looper, LooperSettings};
use crate::dsp::oversample::{Oversampler, OS_LATENCY};
use crate::dsp::prism::{Prism, PrismSettings};
use crate::dsp::ring::{Ring, RingSettings};
use crate::dsp::tuner::Decimator;
use crate::dsp::util::{db_to_gain, Glide, Smoother, Switch, SUB};
use crate::dsp::vibe::{Vibe, VibeSettings};
use crate::shared::Shared;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainSettings {
    pub input_db: f32,
    pub output_db: f32,
    /// Oversampling factor: 1, 2 or 4.
    pub quality: usize,
    pub tuner_mute: bool,
    pub drive: DriveSettings,
    pub fuzz: FuzzSettings,
    pub ring: RingSettings,
    pub vibe: VibeSettings,
    pub prism: PrismSettings,
    pub echo: EchoSettings,
    pub looper: LooperSettings,
    pub loop_btn: bool,
    pub stop_btn: bool,
    pub clear_btn: bool,
}

impl Default for ChainSettings {
    fn default() -> Self {
        Self {
            input_db: 0.0,
            output_db: 0.0,
            quality: 4,
            tuner_mute: false,
            drive: DriveSettings::default(),
            fuzz: FuzzSettings::default(),
            ring: RingSettings::default(),
            vibe: VibeSettings::default(),
            prism: PrismSettings::default(),
            echo: EchoSettings::default(),
            looper: LooperSettings::default(),
            loop_btn: false,
            stop_btn: false,
            clear_btn: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopButton {
    Loop,
    Stop,
    Clear,
}

pub struct Chain {
    sr: f32,
    factor: usize,
    muted: bool,
    os: Oversampler,
    drive: Drive,
    fuzz: Fuzz,
    ring: Ring,
    vibe: Vibe,
    prism: Prism,
    echo: Echo,
    looper: Looper,
    decim: Decimator,
    input: Smoother,
    output: Smoother,
    in_g: Glide,
    out_g: Glide,
    unmute: Switch,
    mute_g: Glide,
}

impl Chain {
    /// Small allocations only. Call `prepare` before processing.
    pub fn new() -> Self {
        Self {
            sr: 48000.0,
            factor: 4,
            muted: false,
            os: Oversampler::new(),
            drive: Drive::new(),
            fuzz: Fuzz::new(),
            ring: Ring::new(),
            vibe: Vibe::new(),
            prism: Prism::new(),
            echo: Echo::new(),
            looper: Looper::new(),
            decim: Decimator::new(),
            input: Smoother::new(0.0),
            output: Smoother::new(0.0),
            in_g: Glide::default(),
            out_g: Glide::default(),
            unmute: Switch::new(true),
            mute_g: Glide::default(),
        }
    }

    /// Allocates all delay lines and the looper. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        self.sr = sr;
        self.prism.prepare(sr);
        self.echo.prepare(sr);
        self.looper.prepare(sr);
        self.vibe.set_sample_rate(sr);
        self.set_os_rate(self.factor);
        self.decim.set_sample_rate(sr);
        self.input.set_time(sr, 20.0);
        self.output.set_time(sr, 20.0);
        self.unmute.set_sample_rate(sr);
        self.reset();
    }

    pub fn latency(&self) -> usize {
        OS_LATENCY
    }

    pub fn tuner_rate(&self) -> f32 {
        self.sr / self.decim.factor() as f32
    }

    fn set_os_rate(&mut self, factor: usize) {
        self.factor = factor;
        let osr = self.sr * factor as f32;
        self.drive.set_sample_rate(osr);
        self.fuzz.set_sample_rate(osr);
        self.ring.set_sample_rate(osr);
    }

    /// Sets all targets (no button handling). Safe on the audio thread.
    pub fn apply(&mut self, s: &ChainSettings) {
        let factor = match s.quality {
            1 => 1,
            2 => 2,
            _ => 4,
        };
        if factor != self.factor {
            self.set_os_rate(factor);
            self.drive.reset();
            self.fuzz.reset();
            self.ring.reset();
        }
        self.drive.set(s.drive);
        self.fuzz.set(s.fuzz);
        self.ring.set(s.ring);
        self.vibe.set(s.vibe);
        self.prism.set(s.prism);
        self.echo.set(s.echo);
        self.looper.set(s.looper);
        self.input.set_target(s.input_db);
        self.output.set_target(s.output_db);
        self.muted = s.tuner_mute;
        self.unmute.set(!s.tuner_mute);
    }

    /// Clears audio state and snaps every smoother to its target.
    pub fn reset(&mut self) {
        self.os.reset();
        self.drive.reset();
        self.fuzz.reset();
        self.ring.reset();
        self.vibe.reset();
        self.prism.reset();
        self.echo.reset();
        self.looper.reset();
        self.decim.set_sample_rate(self.sr);
        let (i, o) = (self.input.target(), self.output.target());
        self.input.reset(i);
        self.output.reset(o);
        self.in_g.hold(db_to_gain(i));
        self.out_g.hold(db_to_gain(o));
        self.unmute.reset(!self.muted);
        self.mute_g.hold(self.unmute.value());
    }

    /// MIDI foot-controller presses (plan.md B9).
    pub fn press(&mut self, b: LoopButton) {
        match b {
            LoopButton::Loop => self.looper.press_loop(),
            LoopButton::Stop => self.looper.press_stop(),
            LoopButton::Clear => self.looper.press_clear(),
        }
    }

    pub fn process(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        s: &ChainSettings,
        shared: &Shared,
    ) {
        self.looper.buttons(s.loop_btn, s.stop_btn, s.clear_btn);
        self.apply(s);
        let n = left.len().min(right.len());
        let mut start = 0;
        while start < n {
            let len = SUB.min(n - start);
            self.process_sub(
                &mut left[start..start + len],
                &mut right[start..start + len],
                shared,
            );
            start += len;
        }
        shared.publish_looper(
            self.looper.state(),
            self.looper.length(),
            self.looper.position(),
        );
    }

    fn process_sub(&mut self, l: &mut [f32], r: &mut [f32], shared: &Shared) {
        let n = l.len();
        // 1. Tuner tap: dry input, before anything touches it.
        for i in 0..n {
            if let Some(v) = self.decim.push(0.5 * (l[i] + r[i])) {
                shared.tuner.push(v);
            }
        }
        // 2. Input gain.
        let (a, b) = self.input.block(n);
        self.in_g.set(db_to_gain(a), db_to_gain(b), n);
        for i in 0..n {
            let g = self.in_g.step();
            l[i] *= g;
            r[i] *= g;
        }
        // 3. Oversampled dirt section.
        let factor = self.factor;
        let frames = n * factor;
        self.drive.begin_block(frames);
        self.fuzz.begin_block(frames);
        self.ring.begin_block(frames);
        let (os, drive, fuzz, ring) = (
            &mut self.os,
            &mut self.drive,
            &mut self.fuzz,
            &mut self.ring,
        );
        os.process(factor, l, r, |a, b| {
            let (a, b) = drive.process_frame(a, b);
            let (a, b) = fuzz.process_frame(a, b);
            ring.process_frame(a, b)
        });
        // 4. Host-rate section.
        self.vibe.begin_block(n);
        self.prism.begin_block(n);
        self.echo.begin_block(n);
        self.looper.begin_block(n);
        let (o0, o1) = self.output.block(n);
        self.out_g.set(db_to_gain(o0), db_to_gain(o1), n);
        let (m0, m1) = self.unmute.block(n);
        self.mute_g.set(m0, m1, n);
        for i in 0..n {
            let (a, b) = self.vibe.process_frame(l[i], r[i]);
            let (a, b) = self.prism.process_frame(a, b);
            let (a, b) = self.echo.process_frame(a, b);
            let (a, b) = self.looper.process_frame(a, b);
            let g = self.out_g.step() * self.mute_g.step();
            l[i] = a * g;
            r[i] = b * g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::looper::LoopState;
    use crate::dsp::testutil::*;
    use crate::dsp::tuner::{Yin, BUFFER};

    const SR: f32 = 48000.0;

    fn all_off(quality: usize) -> ChainSettings {
        let mut s = ChainSettings {
            quality,
            ..Default::default()
        };
        s.drive.on = false;
        s.fuzz.on = false;
        s.ring.on = false;
        s.vibe.on = false;
        s.prism.on = false;
        s.echo.on = false;
        s.echo.trails = false;
        s
    }

    fn make(s: &ChainSettings) -> (Chain, Shared) {
        let mut c = Chain::new();
        c.prepare(SR);
        c.apply(s);
        c.reset();
        (c, Shared::new())
    }

    fn run(c: &mut Chain, sh: &Shared, s: &ChainSettings, x: &[f32]) -> Vec<f32> {
        let mut l = x.to_vec();
        let mut r = x.to_vec();
        for (a, b) in l.chunks_mut(512).zip(r.chunks_mut(512)) {
            c.process(a, b, s, sh);
        }
        l
    }

    #[test]
    fn latency_is_24_samples_at_every_quality() {
        for q in [1usize, 2, 4] {
            let s = all_off(q);
            let (mut c, sh) = make(&s);
            let mut x = vec![0.0f32; 512];
            x[0] = 1.0;
            let y = run(&mut c, &sh, &s, &x);
            assert_eq!(c.latency(), 24);
            assert_eq!(argmax_abs(&y), 24, "quality {q}");
        }
    }

    #[test]
    fn default_board_is_finite_and_bounded() {
        let s = ChainSettings::default();
        let (mut c, sh) = make(&s);
        let y = run(&mut c, &sh, &s, &noise(48000, 0.3, 81));
        assert!(all_finite(&y));
        assert!(peak(&y) < 8.0, "{}", peak(&y));
    }

    #[test]
    fn tuner_mute_silences_output_but_tuner_still_listens() {
        let s = ChainSettings {
            tuner_mute: true,
            ..all_off(4)
        };
        let (mut c, sh) = make(&s);
        let y = run(&mut c, &sh, &s, &sine(440.0, SR, 9600, 0.3));
        assert!(y[1000..].iter().all(|v| *v == 0.0));
        let mut buf = vec![0.0f32; BUFFER];
        sh.tuner.copy_latest(&mut buf);
        let f = Yin::new().detect(&buf, sh.tuner.rate()).expect("pitch");
        assert!((f - 440.0).abs() < 1.0, "{f}");
    }

    #[test]
    fn loop_button_is_published() {
        let s = ChainSettings {
            loop_btn: true,
            ..all_off(1)
        };
        let (mut c, sh) = make(&s);
        run(&mut c, &sh, &s, &vec![0.1; 512]);
        assert_eq!(sh.looper().0, LoopState::Record);
    }
}
