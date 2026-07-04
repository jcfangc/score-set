//! Core implementation for `Score32 = f32`.
//!
//! This module provides the complete scoring framework: define metrics via a
//! builder pipeline, combine them into a [`ScoreSet32`], and produce a closure
//! that evaluates any `&C` context to either a weighted sum or a breakdown.

use alloc::vec::Vec;
use witnessed::{WitnessExt, Witnessed};

use crate::value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};

// ---------------------------------------------------------------------------
// Score32 type alias
// ---------------------------------------------------------------------------

/// The floating-point type used for all scores, weights, and contributions.
pub type Score32 = f32;

// ---------------------------------------------------------------------------
// Map0132 — normalization strategy (data, not closures)
// ---------------------------------------------------------------------------

/// Normalization strategy that maps a raw measure to `[0, 1]`.
///
/// All variants except [`Custom`](Map0132::Custom) guarantee the output is in
/// `[0, 1]` by construction. `Custom` is validated at evaluation time via
/// [`Value01::witness`].
#[derive(Clone, Debug)]
pub enum Map0132 {
    /// Clamp `raw` to `[0, 1]`.
    Identity,
    /// `raw / max`, clamped to `[0, 1]`.
    Linear {
        /// Upper bound for the raw value.
        max: Score32,
    },
    /// Increasing sigmoid: `low → 0`, `high → 1`.
    IncSigmoid {
        /// Lower bound (≈0).
        low: Score32,
        /// Upper bound (≈1).
        high: Score32,
    },
    /// Decreasing sigmoid: `low → 1`, `high → 0`.
    DecSigmoid {
        /// Lower bound (≈1).
        low: Score32,
        /// Upper bound (≈0).
        high: Score32,
    },
    /// Cauchy (Lorentzian) distribution, symmetric about `center`.
    Cauchy {
        /// Peak center.
        center: Score32,
        /// Scale parameter.
        scale: Score32,
    },
    /// User-provided normalization function.
    ///
    /// The function receives the raw measure value and must return a value in
    /// `[0, 1]`. The output is validated at evaluation time.
    Custom(fn(Score32) -> Score32),
}

impl Map0132 {
    /// Apply the normalization to a raw score.
    ///
    /// Returns the normalized value. For `Custom`, the output is validated;
    /// for all other variants correctness is guaranteed by construction.
    #[inline]
    pub fn apply(&self, raw: Score32) -> Result<Witnessed<Score32, Value01>, &'static str> {
        let v = match self {
            Self::Identity => raw.clamp(0.0, 1.0),
            Self::Linear { max } => {
                if *max <= 0.0 {
                    return Err("Map0132::Linear: max must be positive");
                }
                (raw / max).clamp(0.0, 1.0)
            }
            Self::IncSigmoid { low, high } => {
                debug_assert!(high > low, "IncSigmoid: high must exceed low");
                let mid = (low + high) / 2.0;
                let steep = 10.0 / (high - low);
                1.0 / (1.0 + libm::expf(-steep * (raw - mid)))
            }
            Self::DecSigmoid { low, high } => {
                debug_assert!(high > low, "DecSigmoid: high must exceed low");
                let mid = (low + high) / 2.0;
                let steep = 10.0 / (high - low);
                1.0 / (1.0 + libm::expf(steep * (raw - mid)))
            }
            Self::Cauchy { center, scale } => {
                let z = (raw - center) / scale;
                1.0 / (1.0 + z * z)
            }
            Self::Custom(f) => f(raw),
        };
        Value01::witness(v)
    }
}

// ---------------------------------------------------------------------------
// Metric32 — a single compiled scoring unit
// ---------------------------------------------------------------------------

/// A single named scoring metric with its normalization strategy.
///
/// `Metric32<C>` combines a pure measure function `fn(&C) -> Score32` with a
/// [`Map0132`] normalization. It stores no closures that capture state, so
/// [`Vec<Metric32<C>>`] works without trait objects.
pub struct Metric32<C> {
    /// Human-readable name for this metric.
    pub name: &'static str,
    measure: fn(&C) -> Score32,
    map01: Map0132,
}

impl<C> Metric32<C> {
    /// Evaluate this metric against a context.
    ///
    /// Returns the normalized score in `[0, 1]`, witnessed by [`Value01`].
    #[inline]
    pub fn eval(&self, ctx: &C) -> Result<Witnessed<Score32, Value01>, &'static str> {
        let raw = (self.measure)(ctx);
        self.map01.apply(raw)
    }
}

impl<C> Clone for Metric32<C> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            measure: self.measure,
            map01: self.map01.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Metric32 builder pipeline
// ---------------------------------------------------------------------------

/// Entry point for building a [`Metric32`].
///
/// Created by [`metric32`].
pub struct MetricNamingStage32 {
    name: &'static str,
}

impl MetricNamingStage32 {
    /// Transition to the measure stage.
    #[inline]
    pub fn measure(self) -> MeasureStage32 {
        MeasureStage32 { name: self.name }
    }
}

/// Waiting for a measure function.
pub struct MeasureStage32 {
    name: &'static str,
}

impl MeasureStage32 {
    /// Provide the measure function `fn(&C) -> Score32`.
    ///
    /// The function must be a non-capturing closure or fn pointer that extracts
    /// a raw score from the context `C`.
    #[inline]
    pub fn by<C>(self, measure: fn(&C) -> Score32) -> MeasuredStage32<C> {
        MeasuredStage32 {
            name: self.name,
            measure,
        }
    }
}

/// Has a measure function, waiting for a [`Map0132`] strategy.
pub struct MeasuredStage32<C> {
    name: &'static str,
    measure: fn(&C) -> Score32,
}

impl<C> MeasuredStage32<C> {
    /// Transition to the map01 stage.
    #[inline]
    pub fn map01(self) -> Map01Stage32<C> {
        Map01Stage32 {
            name: self.name,
            measure: self.measure,
        }
    }
}

/// Waiting for a normalization strategy.
pub struct Map01Stage32<C> {
    name: &'static str,
    measure: fn(&C) -> Score32,
}

impl<C> Map01Stage32<C> {
    /// Identity normalization: clamps raw to `[0, 1]`.
    #[inline]
    pub fn identity(self) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Identity,
        }
    }

    /// Linear normalization: `raw / max`, clamped to `[0, 1]`.
    #[inline]
    pub fn linear(self, max: Score32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Linear { max },
        }
    }

    /// Increasing sigmoid: `low → ≈0`, `high → ≈1`.
    ///
    /// Uses a logistic curve with steepness `10 / (high - low)`.
    #[inline]
    pub fn inc_sigmoid(self, low: Score32, high: Score32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::IncSigmoid { low, high },
        }
    }

    /// Decreasing sigmoid: `low → ≈1`, `high → ≈0`.
    ///
    /// Uses a logistic curve with steepness `10 / (high - low)`, flipped.
    #[inline]
    pub fn dec_sigmoid(self, low: Score32, high: Score32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::DecSigmoid { low, high },
        }
    }

    /// Cauchy (Lorentzian) normalization.
    ///
    /// The function peaks at `center` and decays symmetrically with `scale`.
    #[inline]
    pub fn cauchy(self, center: Score32, scale: Score32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Cauchy { center, scale },
        }
    }

    /// Custom normalization function.
    ///
    /// The function receives the raw measure value and must return a `[0, 1]`
    /// score. Output is validated via [`Value01::witness`] at evaluation time.
    #[inline]
    pub fn by(self, map01: fn(Score32) -> Score32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Custom(map01),
        }
    }
}

// ---------------------------------------------------------------------------
// Breakdown32 — per-metric detail
// ---------------------------------------------------------------------------

/// A single metric's contribution to the total score.
///
/// Returned by [`Evaluator32::iter`] for a given context `&C`.
#[derive(Clone, Debug)]
pub struct Breakdown32 {
    /// Metric name.
    pub name: &'static str,
    /// Normalized score in `[0, 1]`.
    pub score: Score32,
    /// Normalized weight (sums to 1 across all metrics).
    pub weight: Score32,
    /// `score * weight`.
    pub contribution: Score32,
}

// ---------------------------------------------------------------------------
// BreakdownIter32 — lazy per-metric breakdown iterator
// ---------------------------------------------------------------------------

/// Compile-time evaluator holding normalized weights and metrics.
///
/// Produced by [`ScoreSet32::breakdown`]. Call [`iter`](Evaluator32::iter) to
/// get a lazy [`BreakdownIter32`].
pub struct Evaluator32<C> {
    members: Vec<NormalizedMember32<C>>,
}

impl<C> Evaluator32<C> {
    /// Yield per-metric breakdown rows for the given context.
    ///
    /// Returns a zero-allocation iterator implementing [`ExactSizeIterator`].
    /// Call `.collect()` to get a `Vec<Breakdown32>`.
    #[inline]
    pub fn iter<'a>(&'a self, ctx: &'a C) -> BreakdownIter32<'a, C> {
        BreakdownIter32 {
            inner: self.members.iter(),
            ctx,
        }
    }

    /// Number of metrics in this evaluator.
    #[inline]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Returns true if the evaluator has no metrics.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Lazy iterator over per-metric [`Breakdown32`] rows.
///
/// Produced by [`Evaluator32::iter`]. Zero-allocation — each row is
/// computed on the fly. Implements [`ExactSizeIterator`].
pub struct BreakdownIter32<'a, C> {
    inner: core::slice::Iter<'a, NormalizedMember32<C>>,
    ctx: &'a C,
}

impl<C> Iterator for BreakdownIter32<'_, C> {
    type Item = Breakdown32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let m = self.inner.next()?;
        let score = m
            .metric
            .eval(self.ctx)
            .map(|w| w.into_inner())
            .unwrap_or(0.0);
        let weight = m.weight.into_inner();
        Some(Breakdown32 {
            name: m.metric.name,
            score,
            weight,
            contribution: score * weight,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<C> ExactSizeIterator for BreakdownIter32<'_, C> {}

// ---------------------------------------------------------------------------
// ScoreSet32 — weighted score set builder & closure factory
// ---------------------------------------------------------------------------

/// Builder for a weighted set of [`Metric32`]s.
///
/// `ScoreSet32` collects metrics with raw weights, normalizes them, and produces
/// a closure — either a weighted-sum function or a breakdown iterator.
///
/// # Examples
///
/// ```ignore
/// let scorer = ScoreSet32::new()
///     .push(2.0, gc_metric)?
///     .push(1.0, len_metric)?
///     .sum()?;
///
/// let total: f32 = scorer(&ctx);
/// ```
pub struct ScoreSet32<C> {
    entries: Vec<(Score32, Metric32<C>)>,
}

impl<C> ScoreSet32<C> {
    /// Create an empty score set builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a metric with a raw (unnormalized) weight.
    ///
    /// The weight must be finite and strictly positive. Normalization happens
    /// when [`sum`](Self::sum) or [`breakdown`](Self::breakdown) is called.
    #[inline]
    pub fn push(mut self, weight: Score32, metric: Metric32<C>) -> Result<Self, &'static str> {
        let _validated = GtZero::witness(weight)?;
        self.entries.push((weight, metric));
        Ok(self)
    }

    /// Consume the builder and return a weighted-sum closure.
    ///
    /// Normalizes all weights so they sum to 1, then returns a closure
    /// `impl Fn(&C) -> Score32` that evaluates every metric against the context
    /// and returns the weighted sum.
    ///
    /// # Errors
    ///
    /// Returns an error if the set is empty or if weight normalization fails.
    pub fn sum(self) -> Result<impl Fn(&C) -> Score32, &'static str> {
        let members = self.normalize()?;
        Ok(move |ctx: &C| {
            let mut total: Score32 = 0.0;
            for m in &members {
                if let Ok(score) = m.metric.eval(ctx) {
                    total += score.into_inner() * m.weight.into_inner();
                }
            }
            total
        })
    }

    /// Consume the builder and return an [`Evaluator32`].
    ///
    /// Normalizes all weights so they sum to 1. Call [`Evaluator32::iter`] to
    /// get a lazy, zero-allocation [`BreakdownIter32`] — `.collect()` it to
    /// get a `Vec<Breakdown32>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the set is empty or if weight normalization fails.
    pub fn breakdown(self) -> Result<Evaluator32<C>, &'static str> {
        let members = self.normalize()?;
        Ok(Evaluator32 { members })
    }

    /// Normalize raw weights into a sorted, validated container.
    fn normalize(self) -> Result<Vec<NormalizedMember32<C>>, &'static str> {
        if self.entries.is_empty() {
            return Err("ScoreSet32: must contain at least one metric");
        }

        let raw_weights: Vec<Score32> = self.entries.iter().map(|(w, _)| *w).collect();
        let sum: Score32 = raw_weights.iter().sum();
        let normalized_raw: Vec<Score32> = raw_weights.iter().map(|w| w / sum).collect();

        // Sort a clone for binary search in NormalizedContainer
        let mut sorted = normalized_raw.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

        let container = NormalizedContainer::witness(sorted)?;

        let members: Result<Vec<_>, _> = self
            .entries
            .into_iter()
            .zip(normalized_raw.iter())
            .map(|((_raw_weight, metric), &nw)| {
                let weight = nw
                    .witness()
                    .by(|v| NormalizedWeight::from_normalized_container(*v, &container))?;
                Ok(NormalizedMember32 { weight, metric })
            })
            .collect();

        members
    }
}

impl<C> Default for ScoreSet32<C> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Internal: a metric paired with its normalized, witnessed weight.
struct NormalizedMember32<C> {
    weight: Witnessed<Score32, NormalizedWeight>,
    metric: Metric32<C>,
}

// ---------------------------------------------------------------------------
// Free function: metric32()
// ---------------------------------------------------------------------------

/// Create a new metric with the given name.
///
/// This is the entry point for the metric builder pipeline:
///
/// ```ignore
/// let m = metric32("cleanliness")
///     .measure()
///     .by(|ctx: &Restaurant| ctx.cleanliness)
///     .map01()
///     .linear(100.0);
/// ```
#[inline]
pub fn metric32(name: &'static str) -> MetricNamingStage32 {
    MetricNamingStage32 { name }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests_for_attack;
#[cfg(test)]
mod tests_for_metric;
#[cfg(test)]
mod tests_for_score_set;
