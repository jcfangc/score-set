//! Core implementation for `f32` scoring.
//!
//! This module provides the complete scoring framework: define metrics via a
//! builder pipeline, combine them into a [`ScoreSet32`], and produce a closure
//! that evaluates any `&C` context to either a weighted sum or a breakdown.

use alloc::vec::Vec;
use witnessed::{WitnessExt, Witnessed};

use crate::value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};

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
        max: f32,
    },
    /// Increasing sigmoid: `low → ≈0`, `high → ≈1`.
    ///
    /// Steepness is auto-calibrated: `k = 2·ln(1/ε − 1) / (high − low)` where
    /// `ε = 10·f32::EPSILON`. At `raw = low` output ≈ ε, at `raw = high` ≈ 1−ε.
    IncSigmoid {
        /// Lower bound (≈0).
        low: f32,
        /// Upper bound (≈1).
        high: f32,
    },
    /// Decreasing sigmoid: `low → ≈1`, `high → ≈0`.
    ///
    /// Same auto-calibrated steepness as [`IncSigmoid`](Map0132::IncSigmoid),
    /// with the sign of `k` flipped. At `raw = low` output ≈ 1−ε, at
    /// `raw = high` ≈ ε.
    DecSigmoid {
        /// Lower bound (≈1).
        low: f32,
        /// Upper bound (≈0).
        high: f32,
    },
    /// Asymmetric Cauchy (Lorentzian) with independent left/right half-widths.
    ///
    /// Peaks at `center` with value 1. The half-width at half-maximum is
    /// `half_left` for `raw < center` and `half_right` for `raw >= center`.
    /// When `half_left == half_right` this is the classic symmetric Cauchy.
    Cauchy {
        /// Peak center.
        center: f32,
        /// Half-width at half-maximum for the left side (`raw < center`).
        half_left: f32,
        /// Half-width at half-maximum for the right side (`raw >= center`).
        half_right: f32,
    },
    /// User-provided normalization function.
    ///
    /// The function receives the raw measure value and must return a value in
    /// `[0, 1]`. The output is validated at evaluation time.
    Custom(fn(f32) -> f32),
}

impl Map0132 {
    /// Apply the normalization to a raw score.
    ///
    /// Returns the normalized value. For `Custom`, the output is validated;
    /// for all other variants correctness is guaranteed by construction.
    #[inline]
    pub fn apply(&self, raw: f32) -> Result<Witnessed<f32, Value01>, &'static str> {
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
                let two = 2.0_f32;
                let eps = 10.0 * f32::EPSILON;
                let x0 = (low + high) / two;
                let k = two * libm::logf(1.0 / eps - 1.0) / (high - low);
                1.0 / (1.0 + libm::expf(-k * (raw - x0)))
            }
            Self::DecSigmoid { low, high } => {
                debug_assert!(high > low, "DecSigmoid: high must exceed low");
                let two = 2.0_f32;
                let eps = 10.0 * f32::EPSILON;
                let x0 = (low + high) / two;
                let k = two * libm::logf(1.0 / eps - 1.0) / (high - low);
                1.0 / (1.0 + libm::expf(k * (raw - x0)))
            }
            Self::Cauchy {
                center,
                half_left,
                half_right,
            } => {
                let h = if raw < *center {
                    *half_left
                } else {
                    *half_right
                };
                let z = (raw - center) / h;
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
/// `Metric32<C>` combines a pure measure function `fn(&C) -> f32` with a
/// [`Map0132`] normalization. It stores no closures that capture state, so
/// [`Vec<Metric32<C>>`] works without trait objects.
pub struct Metric32<C> {
    /// Human-readable name for this metric.
    pub name: &'static str,
    measure: fn(&C) -> f32,
    map01: Map0132,
}

impl<C> Metric32<C> {
    /// Evaluate this metric against a context.
    ///
    /// Returns the normalized score in `[0, 1]`, witnessed by [`Value01`].
    #[inline]
    pub fn eval(&self, ctx: &C) -> Result<Witnessed<f32, Value01>, &'static str> {
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
    /// Provide the measure function `fn(&C) -> f32`.
    ///
    /// The function must be a non-capturing closure or fn pointer that extracts
    /// a raw score from the context `C`.
    #[inline]
    pub fn by<C>(self, measure: fn(&C) -> f32) -> MeasuredStage32<C> {
        MeasuredStage32 {
            name: self.name,
            measure,
        }
    }
}

/// Has a measure function, waiting for a [`Map0132`] strategy.
pub struct MeasuredStage32<C> {
    name: &'static str,
    measure: fn(&C) -> f32,
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
    measure: fn(&C) -> f32,
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
    pub fn linear(self, max: f32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Linear { max },
        }
    }

    /// Increasing sigmoid: `low → ≈0`, `high → ≈1`.
    ///
    /// Uses auto-calibrated steepness `k = 2·ln(1/ε − 1) / (high − low)` where
    /// `ε = 10·f32::EPSILON`. At `raw = low` output ≈ ε, at `raw = high` ≈ 1−ε.
    #[inline]
    pub fn inc_sigmoid(self, low: f32, high: f32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::IncSigmoid { low, high },
        }
    }

    /// Decreasing sigmoid: `low → ≈1`, `high → ≈0`.
    ///
    /// Same auto-calibrated steepness as [`inc_sigmoid`](Self::inc_sigmoid),
    /// with the sign flipped.
    #[inline]
    pub fn dec_sigmoid(self, low: f32, high: f32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::DecSigmoid { low, high },
        }
    }

    /// Asymmetric Cauchy (Lorentzian) normalization.
    ///
    /// Peaks at `center` with value 1. `half_left` controls the spread for
    /// `raw < center`, `half_right` for `raw >= center`. When both are equal
    /// this is the classic symmetric Cauchy.
    #[inline]
    pub fn cauchy(self, center: f32, half_left: f32, half_right: f32) -> Metric32<C> {
        Metric32 {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Cauchy {
                center,
                half_left,
                half_right,
            },
        }
    }

    /// Custom normalization function.
    ///
    /// The function receives the raw measure value and must return a `[0, 1]`
    /// score. Output is validated via [`Value01::witness`] at evaluation time.
    #[inline]
    pub fn by(self, map01: fn(f32) -> f32) -> Metric32<C> {
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
/// Returned by the iterator from [`ScoreSet32::breakdown`].
#[derive(Clone, Debug)]
pub struct Breakdown32 {
    /// Metric name.
    pub name: &'static str,
    /// Raw measured value, before [`Map0132`] normalization.
    pub raw: f32,
    /// Normalized score in `[0, 1]`.
    pub score: f32,
    /// Normalized weight (sums to 1 across all metrics).
    pub weight: f32,
    /// `score * weight`.
    pub contribution: f32,
}

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
    entries: Vec<(f32, Metric32<C>)>,
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
    pub fn push(mut self, weight: f32, metric: Metric32<C>) -> Result<Self, &'static str> {
        let _validated = GtZero::witness(weight)?;
        self.entries.push((weight, metric));
        Ok(self)
    }

    /// Consume the builder and return a weighted-sum closure.
    ///
    /// Normalizes all weights so they sum to 1, then returns a closure
    /// `impl Fn(&C) -> f32` that evaluates every metric against the context
    /// and returns the weighted sum.
    ///
    /// # Errors
    ///
    /// Returns an error if the set is empty or if weight normalization fails.
    pub fn sum(self) -> Result<impl Fn(&C) -> f32, &'static str> {
        let members = self.normalize()?;
        Ok(move |ctx: &C| {
            let mut total: f32 = 0.0;
            for m in &members {
                if let Ok(score) = m.metric.eval(ctx) {
                    total += score.into_inner() * m.weight.into_inner();
                }
            }
            total
        })
    }

    /// Consume the builder and return per-metric [`Breakdown32`] rows.
    ///
    /// Normalizes all weights so they sum to 1, evaluates every metric
    /// against `ctx`, and returns the result as `impl IntoIterator`. The
    /// returned value owns all data — no lifetime coupling to `ctx` — so
    /// it can be passed out of local scopes freely.
    ///
    /// Use directly in a `for` loop or call `.into_iter()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the set is empty or if weight normalization fails.
    pub fn breakdown(self, ctx: &C) -> Result<impl IntoIterator<Item = Breakdown32>, &'static str> {
        let members = self.normalize()?;
        Ok(members
            .into_iter()
            .map(|m| {
                let raw = (m.metric.measure)(ctx);
                let score = m
                    .metric
                    .map01
                    .apply(raw)
                    .map(|w| w.into_inner())
                    .unwrap_or(0.0);
                let weight = m.weight.into_inner();
                Breakdown32 {
                    name: m.metric.name,
                    raw,
                    score,
                    weight,
                    contribution: score * weight,
                }
            })
            .collect::<Vec<_>>())
    }

    /// Normalize raw weights into a sorted, validated container.
    fn normalize(self) -> Result<Vec<NormalizedMember32<C>>, &'static str> {
        if self.entries.is_empty() {
            return Err("ScoreSet32: must contain at least one metric");
        }

        let raw_weights: Vec<f32> = self.entries.iter().map(|(w, _)| *w).collect();
        let sum: f32 = raw_weights.iter().sum();
        let normalized_raw: Vec<f32> = raw_weights.iter().map(|w| w / sum).collect();

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
    weight: Witnessed<f32, NormalizedWeight>,
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
