//! Structured breakdown of a [`DynamicScoreSet`](crate::DynamicScoreSet) or
//! [`FiniteScoreSet`](crate::FiniteScoreSet) evaluation — one row per metric.

use crate::float::Float;

/// A single metric's contribution in a score breakdown.
///
/// Returned by [`.breakdown()`](crate::DynamicScoreSet::breakdown). Each item
/// records the metric's name, its raw `[0, 1]` score, its normalized weight,
/// and the resulting weighted contribution (`score × weight`).
///
/// # Fields
///
/// | Field | Type | Description |
/// |---|---|---|
/// | `name` | `&str` | Metric's human-readable name |
/// | `score` | `T` | Raw `[0, 1]` score, **before** weighting |
/// | `weight` | `T` | Normalized weight (`> 0`, sum of all weights = 1) |
/// | `contribution` | `T` | `score × weight` — what this metric added to the total |
pub struct Breakdown<'a, T: Float> {
    /// The metric's human-readable name.
    pub name: &'a str,
    /// The raw `[0, 1]` score from the metric (before weighting).
    pub score: T,
    /// The normalized weight.
    pub weight: T,
    /// The weighted contribution: `score × weight`.
    pub contribution: T,
}
