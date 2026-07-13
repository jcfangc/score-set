//! Core implementation for `f32` scoring.
//!
//! Defines [`Metric32`], [`MetricTuple32`], [`Scorer32`], [`ScoreSet32`],
//! [`Breakdown32`], the builder pipeline, and [`ScorerBuilder32`] (generated into
//! `gen_score_set32.rs`).

use witnessed::Witnessed;

use crate::value::{GtZero, Value01};

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
/// `Metric32<C, F>` combines a measure closure `F: Fn(&C) -> f32` with a
/// [`Map0132`] normalization. The default `F = fn(&C) -> f32` keeps backward
/// compatibility for fn-pointer metrics used with [`ScoreSet32`].
///
/// Use capturing closures for partial application (e.g. thresholds, config
/// parameters), then combine heterogeneous metrics via [`ScorerBuilder32`].
pub struct Metric32<C, F = fn(&C) -> f32> {
    /// Human-readable name for this metric.
    pub name: &'static str,
    measure: F,
    map01: Map0132,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, F: Fn(&C) -> f32> Metric32<C, F> {
    /// Evaluate this metric against a context.
    ///
    /// Returns the normalized score in `[0, 1]`, witnessed by [`Value01`].
    #[inline]
    pub fn eval(&self, ctx: &C) -> Result<Witnessed<f32, Value01>, &'static str> {
        let raw = (self.measure)(ctx);
        self.map01.apply(raw)
    }

    /// Produce a single [`Breakdown32`] row for this metric.
    ///
    /// Evaluates the measure closure and normalization against `ctx`, then
    /// packs the result together with the given `weight` into a breakdown row.
    ///
    /// This is `pub` (not `pub(crate)`) because the per-arity impls in
    /// `gen_score_set32.rs` expand in the caller's crate — `$crate` items
    /// must be fully public to be accessible across crate boundaries.
    #[inline]
    pub fn make_breakdown(&self, weight: f32, ctx: &C) -> Breakdown32 {
        let raw = (self.measure)(ctx);
        let score = self
            .map01
            .apply(raw)
            .map(Witnessed::into_inner)
            .unwrap_or(0.0);
        Breakdown32 {
            name: self.name,
            raw,
            score,
            weight,
            contribution: score * weight,
        }
    }
}

impl<C, F: Clone> Clone for Metric32<C, F> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            measure: self.measure.clone(),
            map01: self.map01.clone(),
            _phantom: core::marker::PhantomData,
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
    /// Provide the measure closure `F: Fn(&C) -> f32`.
    ///
    /// Accepts both function pointers (`fn(&C) -> f32`) and capturing closures.
    /// For use with [`Scorer32`], pass an fn pointer or a non-capturing
    /// closure that coerces to one. For heterogeneous metric types, use
    /// [`ScorerBuilder32`].
    #[inline]
    pub fn by<C, F>(self, measure: F) -> MeasuredStage32<C, F>
    where
        F: Fn(&C) -> f32,
    {
        MeasuredStage32::<C, F> {
            name: self.name,
            measure,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Has a measure function, waiting for a [`Map0132`] strategy.
pub struct MeasuredStage32<C, F = fn(&C) -> f32> {
    name: &'static str,
    measure: F,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, F> MeasuredStage32<C, F> {
    /// Transition to the map01 stage.
    #[inline]
    pub fn map01(self) -> Map01Stage32<C, F> {
        Map01Stage32::<C, F> {
            name: self.name,
            measure: self.measure,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Waiting for a normalization strategy.
pub struct Map01Stage32<C, F = fn(&C) -> f32> {
    name: &'static str,
    measure: F,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, F> Map01Stage32<C, F> {
    /// Identity normalization: clamps raw to `[0, 1]`.
    #[inline]
    pub fn identity(self) -> Metric32<C, F> {
        Metric32::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Identity,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Linear normalization: `raw / max`, clamped to `[0, 1]`.
    #[inline]
    pub fn linear(self, max: f32) -> Metric32<C, F> {
        Metric32::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Linear { max },
            _phantom: core::marker::PhantomData,
        }
    }

    /// Increasing sigmoid: `low → ≈0`, `high → ≈1`.
    ///
    /// Uses auto-calibrated steepness `k = 2·ln(1/ε − 1) / (high − low)` where
    /// `ε = 10·f32::EPSILON`. At `raw = low` output ≈ ε, at `raw = high` ≈ 1−ε.
    #[inline]
    pub fn inc_sigmoid(self, low: f32, high: f32) -> Metric32<C, F> {
        Metric32::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0132::IncSigmoid { low, high },
            _phantom: core::marker::PhantomData,
        }
    }

    /// Decreasing sigmoid: `low → ≈1`, `high → ≈0`.
    ///
    /// Same auto-calibrated steepness as [`inc_sigmoid`](Self::inc_sigmoid),
    /// with the sign flipped.
    #[inline]
    pub fn dec_sigmoid(self, low: f32, high: f32) -> Metric32<C, F> {
        Metric32::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0132::DecSigmoid { low, high },
            _phantom: core::marker::PhantomData,
        }
    }

    /// Asymmetric Cauchy (Lorentzian) normalization.
    ///
    /// Peaks at `center` with value 1. `half_left` controls the spread for
    /// `raw < center`, `half_right` for `raw >= center`. When both are equal
    /// this is the classic symmetric Cauchy.
    #[inline]
    pub fn cauchy(self, center: f32, half_left: f32, half_right: f32) -> Metric32<C, F> {
        Metric32::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Cauchy {
                center,
                half_left,
                half_right,
            },
            _phantom: core::marker::PhantomData,
        }
    }

    /// Custom normalization function.
    ///
    /// The function receives the raw measure value and must return a `[0, 1]`
    /// score. Output is validated via [`Value01::witness`] at evaluation time.
    #[inline]
    pub fn by(self, map01: fn(f32) -> f32) -> Metric32<C, F> {
        Metric32::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0132::Custom(map01),
            _phantom: core::marker::PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Breakdown32 — per-metric detail
// ---------------------------------------------------------------------------

/// A single metric's contribution to the total score.
///
/// Returned by the [`.breakdown()`](ScoreSet32::breakdown) method on a [`Scorer32`].
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
// MetricTuple32 — trait implemented by tuples of Metric32 for batch evaluation
// ---------------------------------------------------------------------------

/// Trait that lets tuples of [`Metric32`]s be iterated and evaluated.
///
/// Implemented for `(Metric32<C, F0>,)`, `(Metric32<C, F0>, Metric32<C, F1>,)`,
/// etc.  Per-arity impls are generated by xtask into `gen_score_set32.rs`.
///
/// This is an internal helper — users interact with [`Scorer32`] and
/// [`ScoreSet32`] instead.
pub trait MetricTuple32<C> {
    /// Compute the weighted sum of all metric scores.
    fn weighted_sum(&self, weights: &[f32], ctx: &C) -> f32;
    /// Collect per-metric [`Breakdown32`] rows.
    fn collect_breakdown(&self, weights: &[f32], ctx: &C) -> alloc::vec::Vec<Breakdown32>;
}

// ---------------------------------------------------------------------------
// ScoreSet32 — public trait for uniform scorer abstraction
// ---------------------------------------------------------------------------

/// Trait for anything that can score a context and produce a breakdown.
///
/// Implemented by [`Scorer32`], enabling `impl ScoreSet32<Ctx>` or
/// `dyn ScoreSet32<Ctx>` for type erasure across different scorer arities.
pub trait ScoreSet32<Ctx> {
    /// Evaluate the weighted sum against a context.
    fn score(&self, ctx: &Ctx) -> f32;
    /// Produce per-metric breakdown rows.
    fn breakdown(&self, ctx: &Ctx) -> alloc::vec::Vec<Breakdown32>;
}

// ---------------------------------------------------------------------------
// Scorer32 — a validated weighted scorer over a flat tuple of metrics
// ---------------------------------------------------------------------------

/// A validated weighted scorer holding a flat tuple of [`Metric32`]s.
///
/// Created via [`ScorerBuilder32::new().add(...).build()`](ScorerBuilder32).
/// Implements [`ScoreSet32`] for evaluating contexts and producing breakdowns.
pub struct Scorer32<C, T: MetricTuple32<C>> {
    metrics: T,
    weights: alloc::vec::Vec<f32>,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, T: MetricTuple32<C>> Scorer32<C, T> {
    /// Build from a tuple of metrics and raw weights (validated).
    ///
    /// Called by [`ScorerBuilder32::build`].  Public because the builder lives
    /// in the generated file and needs cross-module access.
    #[inline]
    pub fn new(metrics: T, raw_weights: &[f32]) -> Result<Self, &'static str> {
        for &w in raw_weights {
            let _ = GtZero::witness(w)?;
        }
        let sum: f32 = raw_weights.iter().sum();
        let weights: alloc::vec::Vec<f32> = raw_weights.iter().map(|w| w / sum).collect();
        Ok(Self {
            metrics,
            weights,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<C, T: MetricTuple32<C>> ScoreSet32<C> for Scorer32<C, T> {
    #[inline]
    fn score(&self, ctx: &C) -> f32 {
        self.metrics.weighted_sum(&self.weights, ctx)
    }
    #[inline]
    fn breakdown(&self, ctx: &C) -> alloc::vec::Vec<Breakdown32> {
        self.metrics.collect_breakdown(&self.weights, ctx)
    }
}

// ---------------------------------------------------------------------------
// ScorerBuilder32 — typestate builder for conditional metric composition
// ---------------------------------------------------------------------------

/// Typestate builder for constructing a [`Scorer32`] metric by metric.
///
/// Start with [`ScorerBuilder32::new()`], call [`.add(weight, metric)`](ScorerBuilder32::add)
/// for each metric, and finish with [`.build()`](ScorerBuilder32::build).
///
/// Each `.add()` changes the type of the builder, enabling zero-vtable static
/// dispatch on the resulting scorer.  Conditional composition is done at the
/// `match` branch level (see crate-level docs).
pub struct ScorerBuilder32<C, T> {
    pub(crate) metrics: T,
    pub(crate) weights: alloc::vec::Vec<f32>,
    pub(crate) _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C> ScorerBuilder32<C, ()> {
    /// Create an empty builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            metrics: (),
            weights: alloc::vec::Vec::new(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<C> Default for ScorerBuilder32<C, ()> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<C, T: MetricTuple32<C>> ScorerBuilder32<C, T> {
    /// Validate weights and build the [`Scorer32`].
    ///
    /// Normalizes weights so they sum to 1.
    ///
    /// # Errors
    /// Returns `Err` if any weight is not strictly positive (`> 0` and finite).
    #[inline]
    pub fn build(self) -> Result<Scorer32<C, T>, &'static str> {
        Scorer32::new(self.metrics, &self.weights)
    }
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
