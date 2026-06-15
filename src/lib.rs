//! # score-set
//!
//! A Rust library for building **weighted scoring operator sets** with three
//! dispatch strategies — from fully static to fully dynamic.
//!
//! It does not prescribe a unified input or context type. Instead it declares,
//! stores, normalizes, and combines a set of weighted operators.
//!
//! # Three-layer architecture
//!
//! | Layer | Type | Dispatch | When to use |
//! |---|---|---|---|
//! | 1 — static | [`ScoreSet`] via [`score_set!`] | Zero vtable, compile-time | Known metric set at compile time |
//! | 2 — enum | [`EnumScoreSet`] via [`declare_metric_enum!`] | Zero vtable (enum match) | Runtime composition, known metric types |
//! | 3 — dynamic | [`DynamicScoreSet`] | Vtable per call | Fully heterogeneous, runtime assembly |
//!
//! # Quick example (Layer 1 — static)
//!
//! ```ignore
//! use score_set::*;
//!
//! let gc = metric("gc")
//!     .measure().by(|dna: &&str| gc_ratio(dna))
//!     .map01().by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());
//!
//! let len = metric("len")
//!     .measure().by(|len: &usize| *len)
//!     .map01().by(|raw: &usize, _: &usize| {
//!         Value01::witness((*raw as f64 / 100.0).min(1.0)).unwrap()
//!     });
//!
//! let ms = score_set! {
//!     2.0 => gc,
//!     3.0 => len,
//! }?;
//!
//! let dna = "ACGTACGT";
//! let score = ms.score().by(|(gc, len)| {
//!     gc.contribute(gc.metric().eval(&dna))
//!         + len.contribute(len.metric().eval(&dna.len()))
//! });
//! # Ok::<(), &'static str>(())
//! ```

mod dynamic_set;
mod enum_set;
mod erased;
mod float;
mod gen_tuple;
mod macros;
mod member;
mod metric;
mod metric_enum;
mod set;
mod value;

// Public API
pub use dynamic_set::{DynamicMember, DynamicScoreSet};
pub use enum_set::{EnumMember, EnumScoreSet};
pub use erased::ErasedMetric;
pub use float::Float;
// score_set! and declare_metric_enum! are exported at crate root via #[macro_export]
pub use member::{Member, Members, RawMember, raw_member};
pub use metric::{Metric, metric};
pub use set::{ScoreSet, ScoreStage};
pub use value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};
pub use witnessed::{WitnessExt, Witnessed};

#[cfg(test)]
mod lab;
