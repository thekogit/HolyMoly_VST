//! All DSP. Nothing in here imports nih_plug, except the enums from crate::params (plan.md B1).
pub mod drive;
pub mod fuzz;
pub mod oversample;
pub mod prism;
pub mod ring;
pub mod util;
pub mod vibe;

#[cfg(test)]
pub mod testutil;
