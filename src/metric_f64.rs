//! Core implementation for `Score64 = f64`.
//!
//! This module provides the complete scoring framework: define metrics via a
//! builder pipeline, combine them into a [`ScoreSet64`], and produce a closure
//! that evaluates any `&C` context to either a weighted sum or a breakdown.

use core::marker::PhantomData;
use witnessed::{WitnessExt, Witnessed};

use crate::value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};

// ---------------------------------------------------------------------------
// Score64 type alias
// ---------------------------------------------------------------------------

/// The floating-point type used for all scores, weights, and contributions.
pub type Score64 = f64;

// ---------------------------------------------------------------------------
// Map0164 — normalization strategy (data, not closures)
// ---------------------------------------------------------------------------

/// Normalization strategy that maps a raw measure to `[0, 1]`.
///
/// All variants except [`Custom`](Map0164::Custom) guarantee the output is in
/// `[0, 1]` by construction. `Custom` is validated at evaluation time via
/// [`Value01::witness`].
#[derive(Clone, Debug)]
pub enum Map0164 {
    /// Clamp `raw` to `[0, 1]`.
    Identity,
    /// `raw / max`, clamped to `[0, 1]`.
    Linear {
        /// Upper bound for the raw value.
        max: Score64,
    },
    /// Increasing sigmoid: `low → 0`, `high → 1`.
    IncSigmoid {
        /// Lower bound (≈0).
        low: Score64,
        /// Upper bound (≈1).
        high: Score64,
    },
    /// Decreasing sigmoid: `low → 1`, `high → 0`.
    DecSigmoid {
        /// Lower bound (≈1).
        low: Score64,
        /// Upper bound (≈0).
        high: Score64,
    },
    /// Cauchy (Lorentzian) distribution, symmetric about `center`.
    Cauchy {
        /// Peak center.
        center: Score64,
        /// Scale parameter.
        scale: Score64,
    },
    /// User-provided normalization function.
    ///
    /// The function receives the raw measure value and must return a value in
    /// `[0, 1]`. The output is validated at evaluation time.
    Custom(fn(Score64) -> Score64),
}

impl Map0164 {
    /// Apply the normalization to a raw score.
    ///
    /// Returns the normalized value. For `Custom`, the output is validated;
    /// for all other variants correctness is guaranteed by construction.
    #[inline]
    pub fn apply(&self, raw: Score64) -> Result<Witnessed<Score64, Value01>, &'static str> {
        let v = match self {
            Self::Identity => raw.clamp(0.0, 1.0),
            Self::Linear { max } => {
                if *max <= 0.0 {
                    return Err("Map0164::Linear: max must be positive");
                }
                (raw / max).clamp(0.0, 1.0)
            }
            Self::IncSigmoid { low, high } => {
                debug_assert!(high > low, "IncSigmoid: high must exceed low");
                let mid = (low + high) / 2.0;
                let steep = 10.0 / (high - low);
                1.0 / (1.0 + (-steep * (raw - mid)).exp())
            }
            Self::DecSigmoid { low, high } => {
                debug_assert!(high > low, "DecSigmoid: high must exceed low");
                let mid = (low + high) / 2.0;
                let steep = 10.0 / (high - low);
                1.0 / (1.0 + (steep * (raw - mid)).exp())
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
// Metric64 — a single compiled scoring unit
// ---------------------------------------------------------------------------

/// A single named scoring metric with its normalization strategy.
///
/// `Metric64<C>` combines a pure measure function `fn(&C) -> Score64` with a
/// [`Map0164`] normalization. It stores no closures that capture state, so
/// [`Vec<Metric64<C>>`] works without trait objects.
pub struct Metric64<C> {
    /// Human-readable name for this metric.
    pub name: String,
    measure: fn(&C) -> Score64,
    map01: Map0164,
}

impl<C> Metric64<C> {
    /// Evaluate this metric against a context.
    ///
    /// Returns the normalized score in `[0, 1]`, witnessed by [`Value01`].
    #[inline]
    pub fn eval(&self, ctx: &C) -> Result<Witnessed<Score64, Value01>, &'static str> {
        let raw = (self.measure)(ctx);
        self.map01.apply(raw)
    }
}

impl<C> Clone for Metric64<C> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            measure: self.measure,
            map01: self.map01.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Metric64 builder pipeline
// ---------------------------------------------------------------------------

/// Entry point for building a [`Metric64`].
///
/// Created by [`metric64`].
pub struct MetricNamingStage64 {
    name: &'static str,
}

impl MetricNamingStage64 {
    /// Transition to the measure stage.
    #[inline]
    pub fn measure(self) -> MeasureStage64 {
        MeasureStage64 { name: self.name }
    }
}

/// Waiting for a measure function.
pub struct MeasureStage64 {
    name: &'static str,
}

impl MeasureStage64 {
    /// Provide the measure function `fn(&C) -> Score64`.
    ///
    /// The function must be a non-capturing closure or fn pointer that extracts
    /// a raw score from the context `C`.
    #[inline]
    pub fn by<C>(self, measure: fn(&C) -> Score64) -> MeasuredStage64<C> {
        MeasuredStage64 {
            name: self.name,
            measure,
            _phantom: PhantomData,
        }
    }
}

/// Has a measure function, waiting for a [`Map0164`] strategy.
pub struct MeasuredStage64<C> {
    name: &'static str,
    measure: fn(&C) -> Score64,
    _phantom: PhantomData<C>,
}

impl<C> MeasuredStage64<C> {
    /// Transition to the map01 stage.
    #[inline]
    pub fn map01(self) -> Map01Stage64<C> {
        Map01Stage64 {
            name: self.name,
            measure: self.measure,
            _phantom: PhantomData,
        }
    }
}

/// Waiting for a normalization strategy.
pub struct Map01Stage64<C> {
    name: &'static str,
    measure: fn(&C) -> Score64,
    _phantom: PhantomData<C>,
}

impl<C> Map01Stage64<C> {
    /// Identity normalization: clamps raw to `[0, 1]`.
    #[inline]
    pub fn identity(self) -> Metric64<C> {
        Metric64 {
            name: self.name.to_string(),
            measure: self.measure,
            map01: Map0164::Identity,
        }
    }

    /// Linear normalization: `raw / max`, clamped to `[0, 1]`.
    #[inline]
    pub fn linear(self, max: Score64) -> Metric64<C> {
        Metric64 {
            name: self.name.to_string(),
            measure: self.measure,
            map01: Map0164::Linear { max },
        }
    }

    /// Increasing sigmoid: `low → ≈0`, `high → ≈1`.
    ///
    /// Uses a logistic curve with steepness `10 / (high - low)`.
    #[inline]
    pub fn inc_sigmoid(self, low: Score64, high: Score64) -> Metric64<C> {
        Metric64 {
            name: self.name.to_string(),
            measure: self.measure,
            map01: Map0164::IncSigmoid { low, high },
        }
    }

    /// Decreasing sigmoid: `low → ≈1`, `high → ≈0`.
    ///
    /// Uses a logistic curve with steepness `10 / (high - low)`, flipped.
    #[inline]
    pub fn dec_sigmoid(self, low: Score64, high: Score64) -> Metric64<C> {
        Metric64 {
            name: self.name.to_string(),
            measure: self.measure,
            map01: Map0164::DecSigmoid { low, high },
        }
    }

    /// Cauchy (Lorentzian) normalization.
    ///
    /// The function peaks at `center` and decays symmetrically with `scale`.
    #[inline]
    pub fn cauchy(self, center: Score64, scale: Score64) -> Metric64<C> {
        Metric64 {
            name: self.name.to_string(),
            measure: self.measure,
            map01: Map0164::Cauchy { center, scale },
        }
    }

    /// Custom normalization function.
    ///
    /// The function receives the raw measure value and must return a `[0, 1]`
    /// score. Output is validated via [`Value01::witness`] at evaluation time.
    #[inline]
    pub fn by(self, map01: fn(Score64) -> Score64) -> Metric64<C> {
        Metric64 {
            name: self.name.to_string(),
            measure: self.measure,
            map01: Map0164::Custom(map01),
        }
    }
}

// ---------------------------------------------------------------------------
// Breakdown64 — per-metric detail
// ---------------------------------------------------------------------------

/// A single metric's contribution to the total score.
///
/// Returned by the breakdown closure produced by [`ScoreSet64::breakdown`].
#[derive(Clone, Debug)]
pub struct Breakdown64 {
    /// Metric name.
    pub name: String,
    /// Normalized score in `[0, 1]`.
    pub score: Score64,
    /// Normalized weight (sums to 1 across all metrics).
    pub weight: Score64,
    /// `score * weight`.
    pub contribution: Score64,
}

// ---------------------------------------------------------------------------
// ScoreSet64 — weighted score set builder & closure factory
// ---------------------------------------------------------------------------

/// Builder for a weighted set of [`Metric64`]s.
///
/// `ScoreSet64` collects metrics with raw weights, normalizes them, and produces
/// a closure — either a weighted-sum function or a breakdown iterator.
///
/// # Examples
///
/// ```ignore
/// let scorer = ScoreSet64::new()
///     .push(2.0, gc_metric)?
///     .push(1.0, len_metric)?
///     .sum()?;
///
/// let total: f64 = scorer(&ctx);
/// ```
pub struct ScoreSet64<C> {
    entries: Vec<(Score64, Metric64<C>)>,
}

impl<C> ScoreSet64<C> {
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
    pub fn push(mut self, weight: Score64, metric: Metric64<C>) -> Result<Self, &'static str> {
        let _validated = GtZero::witness(weight)?;
        self.entries.push((weight, metric));
        Ok(self)
    }

    /// Consume the builder and return a weighted-sum closure.
    ///
    /// Normalizes all weights so they sum to 1, then returns a closure
    /// `impl Fn(&C) -> Score64` that evaluates every metric against the context
    /// and returns the weighted sum.
    ///
    /// # Errors
    ///
    /// Returns an error if the set is empty or if weight normalization fails.
    pub fn sum(self) -> Result<impl Fn(&C) -> Score64, &'static str> {
        let members = self.normalize()?;
        Ok(move |ctx: &C| {
            let mut total: Score64 = 0.0;
            for m in &members {
                if let Ok(score) = m.metric.eval(ctx) {
                    total += score.into_inner() * m.weight.into_inner();
                }
            }
            total
        })
    }

    /// Consume the builder and return a breakdown closure.
    ///
    /// Normalizes all weights so they sum to 1, then returns a closure
    /// `impl Fn(&C) -> Vec<Breakdown64>` that evaluates every metric against
    /// the context and returns per-metric detail rows.
    ///
    /// # Errors
    ///
    /// Returns an error if the set is empty or if weight normalization fails.
    pub fn breakdown(self) -> Result<impl Fn(&C) -> Vec<Breakdown64>, &'static str> {
        let members = self.normalize()?;
        Ok(move |ctx: &C| {
            members
                .iter()
                .map(|m| {
                    let score = m.metric.eval(ctx).map(|w| w.into_inner()).unwrap_or(0.0);
                    let weight = m.weight.into_inner();
                    Breakdown64 {
                        name: m.metric.name.clone(),
                        score,
                        weight,
                        contribution: score * weight,
                    }
                })
                .collect()
        })
    }

    /// Normalize raw weights into a sorted, validated container.
    fn normalize(self) -> Result<Vec<NormalizedMember64<C>>, &'static str> {
        if self.entries.is_empty() {
            return Err("ScoreSet64: must contain at least one metric");
        }

        let raw_weights: Vec<Score64> = self.entries.iter().map(|(w, _)| *w).collect();
        let sum: Score64 = raw_weights.iter().sum();
        let normalized_raw: Vec<Score64> = raw_weights.iter().map(|w| w / sum).collect();

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
                Ok(NormalizedMember64 { weight, metric })
            })
            .collect();

        members
    }
}

impl<C> Default for ScoreSet64<C> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Internal: a metric paired with its normalized, witnessed weight.
struct NormalizedMember64<C> {
    weight: Witnessed<Score64, NormalizedWeight>,
    metric: Metric64<C>,
}

// ---------------------------------------------------------------------------
// Free function: metric64()
// ---------------------------------------------------------------------------

/// Create a new metric with the given name.
///
/// This is the entry point for the metric builder pipeline:
///
/// ```ignore
/// let m = metric64("cleanliness")
///     .measure()
///     .by(|ctx: &Restaurant| ctx.cleanliness)
///     .map01()
///     .linear(100.0);
/// ```
#[inline]
pub fn metric64(name: &'static str) -> MetricNamingStage64 {
    MetricNamingStage64 { name }
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
