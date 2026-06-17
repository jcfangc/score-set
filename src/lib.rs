//! # score-set
//!
//! A Rust library for building **weighted scoring operator sets** with three
//! dispatch strategies — from compile-time fixed to fully dynamic.
//!
//! It does not prescribe a unified input or context type. Instead it declares,
//! stores, normalizes, and combines a set of weighted operators.
//!
//! # Three-layer architecture
//!
//! | Layer | Type | Dispatch | When to use |
//! |---|---|---|---|
//! | 1 — fixed | [`FixedScoreSet`] via [`fixed_score_set!`] | Compile-time, zero vtable | Known metric set at compile time |
//! | 2 — finite | [`FiniteScoreSet`] via [`finite_score_set!`] | Enum match, zero vtable | Runtime composition, known metric types |
//! | 3 — dynamic | [`DynamicScoreSet`] via [`dynamic_score_set!`] | Vtable per call | Fully heterogeneous, runtime assembly |
//!
//! # Quick example (Layer 1 — fixed)
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
//! let ms = fixed_score_set! {
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

mod breakdown;
mod dynamic;
mod finite;
mod fixed;
mod float;
mod gen_tuple;
mod macros;
mod member;
mod metric;
mod value;

// Public API
pub use breakdown::Breakdown;
pub use dynamic::{
    DynMetric, DynamicMember, DynamicScoreSet, DynamicScoreSetBuilder, DynamicScoreStage,
};
pub use finite::{FiniteMember, FiniteScoreSet, FiniteScoreSetBuilder, FiniteScoreStage};
pub use fixed::{FixedScoreSet, ScoreStage};
pub use float::Float;
// fixed_score_set!, dynamic_score_set!, and finite_metric! are exported at crate root via #[macro_export]
pub use member::{Member, Members, RawMember, raw_member};
pub use metric::{Metric, metric};
pub use value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};
pub use witnessed::{WitnessExt, Witnessed};

#[cfg(test)]
mod lab;
