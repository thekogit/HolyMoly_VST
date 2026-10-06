//! State shared between the audio thread and the editor. Lock-free: atomics only.

use crate::dsp::looper::LoopState;
use std::sync::atomic::{AtomicU32, AtomicU8, AtomicUsize, Ordering};

pub const TAP_SIZE: usize = 4096;

/// An f32 stored in an AtomicU32.
pub struct SharedF32(AtomicU32);

impl SharedF32 {
    pub fn new(v: f32) -> Self {
        Self(AtomicU32::new(v.to_bits()))
    }

    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn store(&self, v: f32) {
        self.0.store(v.to_bits(), Ordering::Relaxed)
    }
}

/// Ring of the most recent decimated dry input samples, for the tuner display.
pub struct TunerTap {
    samples: Vec<AtomicU32>,
    write: AtomicUsize,
    rate: SharedF32,
}

impl Default for TunerTap {
    fn default() -> Self {
        Self::new()
    }
}

impl TunerTap {
    pub fn new() -> Self {
        Self {
            samples: (0..TAP_SIZE).map(|_| AtomicU32::new(0)).collect(),
            write: AtomicUsize::new(0),
            rate: SharedF32::new(48000.0),
        }
    }

    /// Audio thread.
    #[inline]
    pub fn push(&self, x: f32) {
        let w = self.write.load(Ordering::Relaxed);
        self.samples[w % TAP_SIZE].store(x.to_bits(), Ordering::Relaxed);
        self.write.store(w.wrapping_add(1), Ordering::Release);
    }

    pub fn set_rate(&self, sr: f32) {
        self.rate.store(sr);
    }

    pub fn rate(&self) -> f32 {
        self.rate.load()
    }

    /// Total samples written so far (wraps).
    pub fn written(&self) -> usize {
        self.write.load(Ordering::Acquire)
    }

    /// Editor thread: copies the newest `out.len()` samples, oldest first.
    pub fn copy_latest(&self, out: &mut [f32]) {
        let w = self.write.load(Ordering::Acquire);
        let n = out.len().min(TAP_SIZE);
        for i in 0..n {
            let idx = w.wrapping_sub(n).wrapping_add(i) % TAP_SIZE;
            out[i] = f32::from_bits(self.samples[idx].load(Ordering::Relaxed));
        }
    }
}

pub struct Shared {
    pub tuner: TunerTap,
    pub sample_rate: SharedF32,
    loop_state: AtomicU8,
    loop_len: AtomicUsize,
    loop_pos: AtomicUsize,
}

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

impl Shared {
    pub fn new() -> Self {
        Self {
            tuner: TunerTap::new(),
            sample_rate: SharedF32::new(48000.0),
            loop_state: AtomicU8::new(0),
            loop_len: AtomicUsize::new(0),
            loop_pos: AtomicUsize::new(0),
        }
    }

    pub fn publish_looper(&self, state: LoopState, len: usize, pos: usize) {
        self.loop_state.store(state.to_u8(), Ordering::Relaxed);
        self.loop_len.store(len, Ordering::Relaxed);
        self.loop_pos.store(pos, Ordering::Relaxed);
    }

    /// (state, length in samples, position in samples).
    pub fn looper(&self) -> (LoopState, usize, usize) {
        (
            LoopState::from_u8(self.loop_state.load(Ordering::Relaxed)),
            self.loop_len.load(Ordering::Relaxed),
            self.loop_pos.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuner_tap_returns_newest_samples_oldest_first() {
        let tap = TunerTap::new();
        for i in 0..5000 {
            tap.push(i as f32);
        }
        let mut out = [0.0f32; 4];
        tap.copy_latest(&mut out);
        assert_eq!(out, [4996.0, 4997.0, 4998.0, 4999.0]);
    }
}
