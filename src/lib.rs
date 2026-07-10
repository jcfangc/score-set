//! # score-set
//!
//! A `#![no_std]` + `alloc` Rust library for building **weighted scoring
//! operators** — define metrics via a builder pipeline, combine them into a
//! zero-vtable flat scorer with the [`score_set32!`] macro.
//!
//! # Quick example
//!
//! ```ignore
//! use score_set::*;
//!
//! struct Restaurant {
//!     cleanliness: f32,
//!     food_quality: f32,
//! }
//!
//! let clean = metric32("cleanliness")
//!     .measure()
//!     .by(|r: &Restaurant| r.cleanliness)
//!     .map01()
//!     .linear(100.0);
//!
//! let food = metric32("food")
//!     .measure()
//!     .by(|r: &Restaurant| r.food_quality)
//!     .map01()
//!     .identity();
//!
//! let scorer = score_set32! { 2.0 => clean, 1.0 => food }?;
//!
//! let r = Restaurant { cleanliness: 80.0, food_quality: 4.0 };
//! let total: f32 = scorer.score(&r);
//! let rows = scorer.breakdown(&r);
//! # Ok::<(), &'static str>(())
//! ```
//!
//! # Partial application
//!
//! [`by()`](MeasureStage32::by) accepts capturing closures, enabling parameter
//! binding at construction time:
//!
//! ```ignore
//! let threshold: f32 = 0.6;
//! let quality = metric32("quality")
//!     .measure()
//!     .by(move |ctx: &MyCtx| if ctx.value > threshold { ctx.value } else { 0.0 })
//!     .map01()
//!     .identity();
//!
//! let scorer = score_set32! { 1.0 => quality }?;
//! // scorer only needs &MyCtx — threshold is baked in
//! ```
//!
//! # no_std
//!
//! This crate is `#![no_std]` with `extern crate alloc` — it only needs
//! `Vec` from the allocator and works on bare-metal targets.

#![no_std]
extern crate alloc;

#[cfg(test)]
extern crate std;

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
pub use value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};
pub use witnessed::{WitnessExt, Witnessed};

// Generated score_set32! macro (f32).  Active by default and in `both` mode.
#[cfg(any(not(feature = "f64"), all(feature = "f64", feature = "both")))]
mod gen_score_set32;

// Generated score_set64! macro (f64).  Active when `f64` or `both` is enabled.
#[cfg(feature = "f64")]
mod gen_score_set64;

#[cfg(test)]
mod test_support;
