//! Moire patterns with no Xous dependencies.
//!
//! * [`patterns`]: layer kinds and rendering into a 1-bit bitmap (XOR of two layers).
//! * [`rects`]: turn a bitmap into few, large rectangles for batched drawing.
//! * [`model`]: menu / preview / gallery state machine driven by decoded keys.

pub mod model;
pub mod patterns;
pub mod rects;

pub use model::{Model, Outcome, Screen};
