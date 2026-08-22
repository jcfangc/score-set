//! A small library for composing weighted scoring metrics into score sets.
//!
//! The crate provides:
//! - `Metric32` / `Metric64` for combining a measure, a normalization map, and a weight.
//! - `DynScoreSet32` / `DynScoreSet64` for storing heterogeneous metrics behind trait objects.

mod dyn_score_set;
mod metric;
pub mod traits;

pub use dyn_score_set::{DynScoreSet32, DynScoreSet32Builder, DynScoreSet64, DynScoreSet64Builder};
pub use metric::{Metric32, Metric64};
pub use traits::{V01Error, prove_v01_f32, prove_v01_f64};
