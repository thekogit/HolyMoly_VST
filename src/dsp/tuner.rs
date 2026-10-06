//! Tuner: `Decimator` (audio thread) feeds the shared tap; `Yin` (editor thread) finds the pitch.

pub const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
pub const WINDOW: usize = 1024;
pub const MAX_LAG: usize = 1024;
pub const BUFFER: usize = WINDOW + MAX_LAG;
const THRESHOLD: f32 = 0.15;
const MIN_RMS: f32 = 0.003;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub freq: f32,
    pub note: usize,
    pub octave: i32,
    pub cents: f32,
}

impl Reading {
    pub fn from_freq(freq: f32, reference: f32) -> Self {
        let midi = 69.0 + 12.0 * (freq / reference).log2();
        let n = midi.round();
        let cents = (midi - n) * 100.0;
        let n = n as i32;
        Self {
            freq,
            note: n.rem_euclid(12) as usize,
            octave: n.div_euclid(12) - 1,
            cents,
        }
    }

    pub fn name(&self) -> &'static str {
        NOTE_NAMES[self.note]
    }
}

/// YIN pitch detector. Allocates in `new`; `detect` does not allocate.
pub struct Yin {
    diff: Vec<f32>,
    cmnd: Vec<f32>,
}

impl Yin {
    pub fn new() -> Self {
        Self {
            diff: vec![0.0; MAX_LAG + 1],
            cmnd: vec![0.0; MAX_LAG + 1],
        }
    }

    /// Uses the newest BUFFER samples of `x` (oldest first). None = silence or no clear pitch.
    pub fn detect(&mut self, x: &[f32], sr: f32) -> Option<f32> {
        if x.len() < BUFFER {
            return None;
        }
        let x = &x[x.len() - BUFFER..];
        let energy = x[..WINDOW].iter().map(|v| v * v).sum::<f32>() / WINDOW as f32;
        if energy.sqrt() < MIN_RMS {
            return None;
        }
        let min_lag = ((sr / 1500.0) as usize).max(2);
        for tau in 1..=MAX_LAG {
            let mut s = 0.0f32;
            for j in 0..WINDOW {
                let d = x[j] - x[j + tau];
                s += d * d;
            }
            self.diff[tau] = s;
        }
        self.cmnd[0] = 1.0;
        let mut running = 0.0f32;
        for tau in 1..=MAX_LAG {
            running += self.diff[tau];
            self.cmnd[tau] = if running > 0.0 {
                self.diff[tau] * tau as f32 / running
            } else {
                1.0
            };
        }
        let mut tau = min_lag;
        let mut found = None;
        while tau < MAX_LAG {
            if self.cmnd[tau] < THRESHOLD {
                while tau + 1 < MAX_LAG && self.cmnd[tau + 1] < self.cmnd[tau] {
                    tau += 1;
                }
                found = Some(tau);
                break;
            }
            tau += 1;
        }
        let tau = found?;
        let (a, b, c) = (self.cmnd[tau - 1], self.cmnd[tau], self.cmnd[tau + 1]);
        let denom = a - 2.0 * b + c;
        let shift = if denom.abs() > 1e-12 {
            0.5 * (a - c) / denom
        } else {
            0.0
        };
        Some(sr / (tau as f32 + shift.clamp(-1.0, 1.0)))
    }
}

/// Averages groups of samples so the tuner always sees about 48 kHz.
pub struct Decimator {
    factor: usize,
    acc: f32,
    count: usize,
}

impl Decimator {
    pub fn new() -> Self {
        Self {
            factor: 1,
            acc: 0.0,
            count: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.factor = ((sr / 48000.0).round() as usize).max(1);
        self.acc = 0.0;
        self.count = 0;
    }

    pub fn factor(&self) -> usize {
        self.factor
    }

    #[inline]
    pub fn push(&mut self, x: f32) -> Option<f32> {
        self.acc += x;
        self.count += 1;
        if self.count == self.factor {
            let y = self.acc / self.factor as f32;
            self.acc = 0.0;
            self.count = 0;
            Some(y)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;

    const SR: f32 = 48000.0;

    fn detect(x: &[f32]) -> Option<f32> {
        Yin::new().detect(x, SR)
    }

    #[test]
    fn finds_low_e() {
        let f = detect(&sine(82.41, SR, BUFFER, 0.3)).expect("pitch");
        let r = Reading::from_freq(f, 440.0);
        assert_eq!(r.name(), "E");
        assert_eq!(r.octave, 2);
        assert!(r.cents.abs() < 2.0, "{}", r.cents);
    }

    #[test]
    fn finds_a4() {
        let f = detect(&sine(440.0, SR, BUFFER, 0.3)).expect("pitch");
        let r = Reading::from_freq(f, 440.0);
        assert_eq!(r.name(), "A");
        assert_eq!(r.octave, 4);
        assert!(r.cents.abs() < 2.0, "{}", r.cents);
    }

    #[test]
    fn cents_follow_the_reference() {
        let f = detect(&sine(445.0, SR, BUFFER, 0.3)).expect("pitch");
        let r = Reading::from_freq(f, 440.0);
        assert!((r.cents - 19.56).abs() < 2.0, "{}", r.cents);
        let r432 = Reading::from_freq(432.0, 432.0);
        assert_eq!(r432.name(), "A");
        assert!(r432.cents.abs() < 0.01);
    }

    #[test]
    fn harmonics_do_not_cause_octave_errors() {
        let mut x = vec![0.0f32; BUFFER];
        for k in 1..=8 {
            let h = sine(110.0 * k as f32, SR, BUFFER, 0.3 / k as f32);
            for i in 0..BUFFER {
                x[i] += h[i];
            }
        }
        let r = Reading::from_freq(detect(&x).expect("pitch"), 440.0);
        assert_eq!(r.name(), "A");
        assert_eq!(r.octave, 2);
    }

    #[test]
    fn silence_gives_no_reading() {
        assert!(detect(&noise(BUFFER, 0.001, 71)).is_none());
        assert!(detect(&vec![0.0; 100]).is_none());
    }

    #[test]
    fn decimator_matches_rates() {
        let mut d = Decimator::new();
        d.set_sample_rate(96000.0);
        assert_eq!(d.factor(), 2);
        assert_eq!(d.push(1.0), None);
        assert_eq!(d.push(3.0), Some(2.0));
        d.set_sample_rate(44100.0);
        assert_eq!(d.factor(), 1);
        d.set_sample_rate(192000.0);
        assert_eq!(d.factor(), 4);
    }
}
