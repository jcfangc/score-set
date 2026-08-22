//! A small library for composing weighted scoring metrics into score sets.
//!
//! The crate provides:
//! - `NormalizedEval32` / `NormalizedEval64` for combining a measure and normalization map.
//! - `Weighted32` / `Weighted64` for weighting any evaluator.
//! - `DynScoreSet32` / `DynScoreSet64` for storing heterogeneous metrics behind trait objects.

#![no_std]

extern crate alloc;

mod dyn_score_set;
mod metric;
mod normalized_eval;
pub mod traits;
mod weighted;

pub use dyn_score_set::{DynScoreSet32, DynScoreSet32Builder, DynScoreSet64, DynScoreSet64Builder};
pub use metric::{metric32, metric64};
pub use normalized_eval::{NormalizedEval32, NormalizedEval64};
pub use traits::{V01Error, prove_v01_f32, prove_v01_f64};
pub use weighted::{Weighted32, Weighted64, weight32, weight64};
