//! # score-set
//!
//! A Rust library for building **static weighted scoring operator sets**.
//!
//! It does not prescribe a unified input or context type. Instead it declares,
//! stores, normalizes, and combines a set of weighted operators — scoring is
//! done via a user-provided closure that can inject arbitrary runtime data.
//!
//! # Quick example
//!
//! ```ignore
//! use score_set::*;
//!
//! let gc = metric("gc")
//!     .measure().by(|dna: &str| gc_ratio(dna))
//!     .map01().by(|raw: &f64, _: &str| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
//!     .build();
//!
//! let len = metric("len")
//!     .measure().by(|len: usize| len)
//!     .map01().by(|raw: &usize, _: usize| {
//!         ((*raw as f64 / 100.0).min(1.0)).witness().by(|v| Value01::prove(*v)).unwrap()
//!     })
//!     .build();
//!
//! let ms = score_set! {
//!     2.0 => gc,
//!     3.0 => len,
//! }.aggregate(strategy::weighted_mean)?;
//!
//! let dna = "ACGTACGT";
//! let score = ms.score().by(|(gc, len)| {
//!     gc.contribute(gc.metric().eval(dna))
//!         + len.contribute(len.metric().eval(dna.len()))
//! });
//! # Ok::<(), &'static str>(())
//! ```

mod float;
mod gen_tuple;
mod macros;
mod member;
mod metric;
mod op;
mod set;
pub mod strategy;
mod value;

// Public API
pub use float::ScoreFloat;
// score_set! is exported at crate root via #[macro_export]
pub use member::{Member, Members, RawMember, raw_member};
pub use metric::{Metric, metric};
pub use op::{Op, op};
pub use set::{RawScoreSet, ScoreSet, ScoreStage};
pub use value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};
pub use witnessed::{WitnessExt, Witnessed};

#[cfg(test)]
mod lab;
