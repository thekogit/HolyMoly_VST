//! Helpers for unit tests only.

use std::f64::consts::PI;

pub fn sine(freq: f32, sr: f32, len: usize, amp: f32) -> Vec<f32> {
    (0..len)
        .map(|n| (amp as f64 * (2.0 * PI * freq as f64 * n as f64 / sr as f64).sin()) as f32)
        .collect()
}

pub fn noise(len: usize, amp: f32, seed: u32) -> Vec<f32> {
    let mut rng = crate::dsp::util::Rng::new(seed);
    (0..len).map(|_| rng.next_f32() * amp).collect()
}

/// Amplitude of the `freq` component (single-bin DFT). Use whole numbers of cycles for exact results.
pub fn tone_amplitude(x: &[f32], freq: f32, sr: f32) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    let w = 2.0 * PI * freq as f64 / sr as f64;
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    for (n, &v) in x.iter().enumerate() {
        re += v as f64 * (w * n as f64).cos();
        im -= v as f64 * (w * n as f64).sin();
    }
    (2.0 * (re * re + im * im).sqrt() / x.len() as f64) as f32
}

pub fn rms(x: &[f32]) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    (x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64).sqrt() as f32
}

pub fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0f32, |m, v| m.max(v.abs()))
}

pub fn all_finite(x: &[f32]) -> bool {
    x.iter().all(|v| v.is_finite())
}

pub fn argmax_abs(x: &[f32]) -> usize {
    let mut best = 0;
    for i in 0..x.len() {
        if x[i].abs() > x[best].abs() {
            best = i;
        }
    }
    best
}

pub fn db(x: f32) -> f32 {
    20.0 * x.abs().max(1e-12).log10()
}
