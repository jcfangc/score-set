//! # score-set
//!
//! A Rust library for building **weighted scoring operators** as composable
//! closures.
//!
//! Define metrics via a builder pipeline, combine them with weights, and
//! produce a closure — either a weighted sum function or a breakdown iterator.
//! The downstream caller only sees `impl Fn(&C) -> Score` or
//! `impl Fn(&C) -> Vec<Breakdown>`.
//!
//! # Quick example
//!
//! ```ignore
//! use score_set::*;
//!
//! struct DnaCtx<'a> {
//!     dna: &'a str,
//!     len: usize,
//! }
//!
//! let gc = metric32("gc")
//!     .measure()
//!     .by(|ctx: &DnaCtx| gc_ratio(ctx.dna) as f32)
//!     .map01()
//!     .identity();
//!
//! let len = metric32("len")
//!     .measure()
//!     .by(|ctx: &DnaCtx| ctx.len as f32)
//!     .map01()
//!     .linear(100.0);
//!
//! let scorer = ScoreSet32::new()
//!     .push(2.0, gc)?
//!     .push(1.0, len)?
//!     .sum()?;
//!
//! let ctx = DnaCtx { dna: "ACGTACGT", len: 8 };
//! let total = scorer(&ctx);
//! # Ok::<(), &'static str>(())
//! ```

// Three-state precision selection:
//   default        → f32  only (ScoreSet32, Metric32, … at crate root)
//   f64            → f64  only (ScoreSet64, Metric64, … at crate root)
//   f64 + both     → f32 + f64 (ScoreSet32 + ScoreSet64 at crate root)
// All modules are private — users only depend on re-exported types.

#[cfg(not(feature = "f64"))]
mod metric_f32;
#[cfg(not(feature = "f64"))]
pub use metric_f32::*;

#[cfg(all(feature = "f64", not(feature = "both")))]
mod metric_f64;
#[cfg(all(feature = "f64", not(feature = "both")))]
pub use metric_f64::*;

#[cfg(all(feature = "f64", feature = "both"))]
mod metric_f32;
#[cfg(all(feature = "f64", feature = "both"))]
mod metric_f64;
#[cfg(all(feature = "f64", feature = "both"))]
pub use metric_f32::*;
#[cfg(all(feature = "f64", feature = "both"))]
pub use metric_f64::*;

mod value;
pub use value::{GtZero, NormalizedContainer, NormalizedWeight, ScoreOps, Value01};
pub use witnessed::{WitnessExt, Witnessed};

#[cfg(test)]
mod test_support;
