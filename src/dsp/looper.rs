//! Looper (after the Hazarai): 30 s stereo, one-button cycle Record -> Play -> Overdub -> Play.
//! Buttons act on their rising edge (plan.md B10). MIDI notes call `press_*` directly.

use crate::dsp::util::{db_to_gain, Glide, Smoother};

pub const MAX_LOOP_SECONDS: f32 = 30.0;
const FADE: usize = 96;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopState {
    Empty,
    Record,
    Play,
    Overdub,
    Stopped,
}

impl LoopState {
    pub fn to_u8(self) -> u8 {
        match self {
            LoopState::Empty => 0,
            LoopState::Record => 1,
            LoopState::Play => 2,
            LoopState::Overdub => 3,
            LoopState::Stopped => 4,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => LoopState::Record,
            2 => LoopState::Play,
            3 => LoopState::Overdub,
            4 => LoopState::Stopped,
            _ => LoopState::Empty,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LooperSettings {
    pub level_db: f32,
    pub reverse: bool,
}

impl Default for LooperSettings {
    fn default() -> Self {
        Self {
            level_db: 0.0,
            reverse: false,
        }
    }
}

/// Playback gain; -60 dB and below is silence.
pub fn level_gain(db: f32) -> f32 {
    if db <= -59.95 {
        0.0
    } else {
        db_to_gain(db)
    }
}

pub struct Looper {
    buf: [Vec<f32>; 2],
    cap: usize,
    len: usize,
    pos: usize,
    state: LoopState,
    s: LooperSettings,
    level: Smoother,
    level_g: Glide,
    prev: [bool; 3],
}

impl Looper {
    pub fn new() -> Self {
        Self {
            buf: [Vec::new(), Vec::new()],
            cap: 0,
            len: 0,
            pos: 0,
            state: LoopState::Empty,
            s: LooperSettings::default(),
            level: Smoother::new(1.0),
            level_g: Glide::default(),
            prev: [false; 3],
        }
    }

    /// Allocates 30 s per channel and clears the loop. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        self.cap = (MAX_LOOP_SECONDS * sr) as usize;
        for b in self.buf.iter_mut() {
            *b = vec![0.0; self.cap];
        }
        self.level.set_time(sr, 20.0);
        self.len = 0;
        self.pos = 0;
        self.state = LoopState::Empty;
        self.reset();
    }

    pub fn set(&mut self, s: LooperSettings) {
        self.s = s;
        self.level.set_target(level_gain(s.level_db));
    }

    /// Keeps the loop, stops playback.
    pub fn reset(&mut self) {
        match self.state {
            LoopState::Record => {
                self.finish_recording();
                self.state = if self.len > 0 {
                    LoopState::Stopped
                } else {
                    LoopState::Empty
                };
            }
            LoopState::Play | LoopState::Overdub => self.state = LoopState::Stopped,
            _ => {}
        }
        self.pos = self.start_pos();
        let g = level_gain(self.s.level_db);
        self.level.reset(g);
        self.level_g.hold(g);
    }

    pub fn state(&self) -> LoopState {
        self.state
    }

    pub fn length(&self) -> usize {
        self.len
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    fn start_pos(&self) -> usize {
        if self.s.reverse && self.len > 0 {
            self.len - 1
        } else {
            0
        }
    }

    fn finish_recording(&mut self) {
        self.len = self.pos.min(self.cap);
        let f = FADE.min(self.len / 4);
        for i in 0..f {
            let g = i as f32 / f as f32;
            let end = self.len - 1 - i;
            for b in self.buf.iter_mut() {
                b[i] *= g;
                b[end] *= g;
            }
        }
        self.pos = self.start_pos();
    }

    /// Rising-edge detection for the three button parameters.
    pub fn buttons(&mut self, loop_btn: bool, stop: bool, clear: bool) {
        if loop_btn && !self.prev[0] {
            self.press_loop();
        }
        if stop && !self.prev[1] {
            self.press_stop();
        }
        if clear && !self.prev[2] {
            self.press_clear();
        }
        self.prev = [loop_btn, stop, clear];
    }

    pub fn press_loop(&mut self) {
        match self.state {
            LoopState::Empty => {
                if self.cap > 0 {
                    self.state = LoopState::Record;
                    self.pos = 0;
                    self.len = 0;
                }
            }
            LoopState::Record => {
                self.finish_recording();
                self.state = if self.len > 0 {
                    LoopState::Play
                } else {
                    LoopState::Empty
                };
            }
            LoopState::Play => self.state = LoopState::Overdub,
            LoopState::Overdub => self.state = LoopState::Play,
            LoopState::Stopped => {
                self.state = LoopState::Play;
                self.pos = self.start_pos();
            }
        }
    }

    pub fn press_stop(&mut self) {
        match self.state {
            LoopState::Record => {
                self.finish_recording();
                self.state = if self.len > 0 {
                    LoopState::Stopped
                } else {
                    LoopState::Empty
                };
            }
            LoopState::Play | LoopState::Overdub => {
                self.state = LoopState::Stopped;
                self.pos = self.start_pos();
            }
            _ => {}
        }
    }

    pub fn press_clear(&mut self) {
        self.state = LoopState::Empty;
        self.len = 0;
        self.pos = 0;
    }

    pub fn begin_block(&mut self, frames: usize) {
        let (g0, g1) = self.level.block(frames);
        self.level_g.set(g0, g1, frames);
    }

    fn advance(&mut self) {
        if self.len == 0 {
            return;
        }
        if self.s.reverse {
            self.pos = if self.pos == 0 {
                self.len - 1
            } else {
                self.pos - 1
            };
        } else {
            self.pos += 1;
            if self.pos >= self.len {
                self.pos = 0;
            }
        }
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let g = self.level_g.step();
        match self.state {
            LoopState::Empty | LoopState::Stopped => (l, r),
            LoopState::Record => {
                if self.pos < self.cap {
                    self.buf[0][self.pos] = l;
                    self.buf[1][self.pos] = r;
                    self.pos += 1;
                }
                if self.pos >= self.cap {
                    self.finish_recording();
                    self.state = LoopState::Play;
                }
                (l, r)
            }
            LoopState::Play | LoopState::Overdub => {
                let i = self.pos;
                let a = self.buf[0][i];
                let b = self.buf[1][i];
                if self.state == LoopState::Overdub {
                    self.buf[0][i] = a + l;
                    self.buf[1][i] = b + r;
                }
                self.advance();
                (l + a * g, r + b * g)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 1000.0;

    fn make() -> Looper {
        let mut lp = Looper::new();
        lp.prepare(SR);
        lp
    }

    fn run(lp: &mut Looper, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            lp.begin_block(chunk.len());
            for &v in chunk {
                out.push(lp.process_frame(v, v).0);
            }
        }
        out
    }

    fn recorded(lp: &mut Looper, x: &[f32]) {
        lp.press_loop();
        run(lp, x);
        lp.press_loop();
    }

    #[test]
    fn empty_is_exact_passthrough() {
        let mut lp = make();
        let x = noise(500, 0.5, 61);
        assert_eq!(run(&mut lp, &x), x);
    }

    #[test]
    fn record_then_play_repeats_the_loop() {
        let mut lp = make();
        let x = noise(1000, 0.5, 62);
        recorded(&mut lp, &x);
        assert_eq!(lp.state(), LoopState::Play);
        assert_eq!(lp.length(), 1000);
        let y = run(&mut lp, &vec![0.0; 1000]);
        for i in 100..900 {
            assert!((y[i] - x[i]).abs() < 1e-6, "{i}");
        }
    }

    #[test]
    fn overdub_adds_to_the_loop() {
        let mut lp = make();
        let a = noise(1000, 0.3, 63);
        let b = noise(1000, 0.3, 64);
        recorded(&mut lp, &a);
        lp.press_loop();
        assert_eq!(lp.state(), LoopState::Overdub);
        run(&mut lp, &b);
        lp.press_loop();
        assert_eq!(lp.state(), LoopState::Play);
        let y = run(&mut lp, &vec![0.0; 1000]);
        for i in 100..900 {
            assert!((y[i] - (a[i] + b[i])).abs() < 1e-5, "{i}");
        }
    }

    #[test]
    fn stop_then_loop_resumes_from_the_start() {
        let mut lp = make();
        let x = noise(1000, 0.5, 65);
        recorded(&mut lp, &x);
        run(&mut lp, &vec![0.0; 300]);
        lp.press_stop();
        assert_eq!(lp.state(), LoopState::Stopped);
        let silent = run(&mut lp, &vec![0.0; 200]);
        assert!(silent.iter().all(|v| *v == 0.0));
        lp.press_loop();
        assert_eq!(lp.state(), LoopState::Play);
        let y = run(&mut lp, &vec![0.0; 1000]);
        for i in 100..900 {
            assert!((y[i] - x[i]).abs() < 1e-6, "{i}");
        }
    }

    #[test]
    fn clear_empties_the_loop() {
        let mut lp = make();
        recorded(&mut lp, &noise(1000, 0.5, 66));
        lp.press_clear();
        assert_eq!(lp.state(), LoopState::Empty);
        assert_eq!(lp.length(), 0);
        let x = noise(300, 0.5, 67);
        assert_eq!(run(&mut lp, &x), x);
    }

    #[test]
    fn reverse_plays_backwards() {
        let mut lp = make();
        let ramp: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
        lp.set(LooperSettings {
            level_db: 0.0,
            reverse: true,
        });
        recorded(&mut lp, &ramp);
        let y = run(&mut lp, &vec![0.0; 1000]);
        assert!(
            y[200] > y[300] && y[300] > y[400],
            "{} {} {}",
            y[200],
            y[300],
            y[400]
        );
    }

    #[test]
    fn level_minus_60_is_silent() {
        let mut lp = make();
        recorded(&mut lp, &noise(1000, 0.5, 68));
        lp.set(LooperSettings {
            level_db: -60.0,
            reverse: false,
        });
        let y = run(&mut lp, &vec![0.0; 1000]);
        assert!(y[200..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn recording_stops_at_30_seconds() {
        let mut lp = make();
        lp.press_loop();
        run(&mut lp, &vec![0.1; 31000]);
        assert_eq!(lp.state(), LoopState::Play);
        assert_eq!(lp.length(), 30000);
    }

    #[test]
    fn buttons_act_on_rising_edges() {
        let mut lp = make();
        lp.buttons(true, false, false);
        assert_eq!(lp.state(), LoopState::Record);
        lp.buttons(true, false, false);
        assert_eq!(lp.state(), LoopState::Record);
        run(&mut lp, &vec![0.1; 100]);
        lp.buttons(false, false, false);
        lp.buttons(true, false, false);
        assert_eq!(lp.state(), LoopState::Play);
    }
}
