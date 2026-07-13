//! Core implementation for `f64` scoring.
//!
//! Defines [`Metric64`], [`MetricTuple64`], [`Scorer64`], [`ScoreSet64`],
//! [`Breakdown64`], the builder pipeline, and [`ScorerBuilder64`] (generated into
//! `gen_score_set64.rs`).

use witnessed::Witnessed;

use crate::value::{GtZero, Value01};

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
        max: f64,
    },
    /// Increasing sigmoid: `low → ≈0`, `high → ≈1`.
    ///
    /// Steepness is auto-calibrated: `k = 2·ln(1/ε − 1) / (high − low)` where
    /// `ε = 10·f64::EPSILON`. At `raw = low` output ≈ ε, at `raw = high` ≈ 1−ε.
    IncSigmoid {
        /// Lower bound (≈0).
        low: f64,
        /// Upper bound (≈1).
        high: f64,
    },
    /// Decreasing sigmoid: `low → ≈1`, `high → ≈0`.
    ///
    /// Same auto-calibrated steepness as [`IncSigmoid`](Map0164::IncSigmoid),
    /// with the sign of `k` flipped. At `raw = low` output ≈ 1−ε, at
    /// `raw = high` ≈ ε.
    DecSigmoid {
        /// Lower bound (≈1).
        low: f64,
        /// Upper bound (≈0).
        high: f64,
    },
    /// Asymmetric Cauchy (Lorentzian) with independent left/right half-widths.
    ///
    /// Peaks at `center` with value 1. The half-width at half-maximum is
    /// `half_left` for `raw < center` and `half_right` for `raw >= center`.
    /// When `half_left == half_right` this is the classic symmetric Cauchy.
    Cauchy {
        /// Peak center.
        center: f64,
        /// Half-width at half-maximum for the left side (`raw < center`).
        half_left: f64,
        /// Half-width at half-maximum for the right side (`raw >= center`).
        half_right: f64,
    },
    /// User-provided normalization function.
    ///
    /// The function receives the raw measure value and must return a value in
    /// `[0, 1]`. The output is validated at evaluation time.
    Custom(fn(f64) -> f64),
}

impl Map0164 {
    /// Apply the normalization to a raw score.
    ///
    /// Returns the normalized value. For `Custom`, the output is validated;
    /// for all other variants correctness is guaranteed by construction.
    #[inline]
    pub fn apply(&self, raw: f64) -> Result<Witnessed<f64, Value01>, &'static str> {
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
                let two = 2.0_f64;
                let eps = 10.0 * f64::EPSILON;
                let x0 = (low + high) / two;
                let k = two * libm::log(1.0 / eps - 1.0) / (high - low);
                1.0 / (1.0 + libm::exp(-k * (raw - x0)))
            }
            Self::DecSigmoid { low, high } => {
                debug_assert!(high > low, "DecSigmoid: high must exceed low");
                let two = 2.0_f64;
                let eps = 10.0 * f64::EPSILON;
                let x0 = (low + high) / two;
                let k = two * libm::log(1.0 / eps - 1.0) / (high - low);
                1.0 / (1.0 + libm::exp(k * (raw - x0)))
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
// Metric64 — a single compiled scoring unit
// ---------------------------------------------------------------------------

/// A single named scoring metric with its normalization strategy.
///
/// `Metric64<C, F>` combines a measure closure `F: Fn(&C) -> f64` with a
/// [`Map0164`] normalization. The default `F = fn(&C) -> f64` keeps backward
/// compatibility for fn-pointer metrics used with [`ScoreSet64`].
///
/// Use capturing closures for partial application (e.g. thresholds, config
/// parameters), then combine heterogeneous metrics via [`ScorerBuilder64`].
pub struct Metric64<C, F = fn(&C) -> f64> {
    /// Human-readable name for this metric.
    pub name: &'static str,
    measure: F,
    map01: Map0164,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, F: Fn(&C) -> f64> Metric64<C, F> {
    /// Evaluate this metric against a context.
    ///
    /// Returns the normalized score in `[0, 1]`, witnessed by [`Value01`].
    #[inline]
    pub fn eval(&self, ctx: &C) -> Result<Witnessed<f64, Value01>, &'static str> {
        let raw = (self.measure)(ctx);
        self.map01.apply(raw)
    }

    /// Produce a single [`Breakdown64`] row for this metric.
    ///
    /// Evaluates the measure closure and normalization against `ctx`, then
    /// packs the result together with the given `weight` into a breakdown row.
    ///
    /// This is `pub` (not `pub(crate)`) because the per-arity impls in
    /// `gen_score_set64.rs` expand in the caller's crate — `$crate` items
    /// must be fully public to be accessible across crate boundaries.
    #[inline]
    pub fn make_breakdown(&self, weight: f64, ctx: &C) -> Breakdown64 {
        let raw = (self.measure)(ctx);
        let score = self
            .map01
            .apply(raw)
            .map(Witnessed::into_inner)
            .unwrap_or(0.0);
        Breakdown64 {
            name: self.name,
            raw,
            score,
            weight,
            contribution: score * weight,
        }
    }
}

impl<C, F: Clone> Clone for Metric64<C, F> {
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
    /// Provide the measure closure `F: Fn(&C) -> f64`.
    ///
    /// Accepts both function pointers (`fn(&C) -> f64`) and capturing closures.
    /// For use with [`Scorer64`], pass an fn pointer or a non-capturing
    /// closure that coerces to one. For heterogeneous metric types, use
    /// [`ScorerBuilder64`].
    #[inline]
    pub fn by<C, F>(self, measure: F) -> MeasuredStage64<C, F>
    where
        F: Fn(&C) -> f64,
    {
        MeasuredStage64::<C, F> {
            name: self.name,
            measure,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Has a measure function, waiting for a [`Map0164`] strategy.
pub struct MeasuredStage64<C, F = fn(&C) -> f64> {
    name: &'static str,
    measure: F,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, F> MeasuredStage64<C, F> {
    /// Transition to the map01 stage.
    #[inline]
    pub fn map01(self) -> Map01Stage64<C, F> {
        Map01Stage64::<C, F> {
            name: self.name,
            measure: self.measure,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Waiting for a normalization strategy.
pub struct Map01Stage64<C, F = fn(&C) -> f64> {
    name: &'static str,
    measure: F,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, F> Map01Stage64<C, F> {
    /// Identity normalization: clamps raw to `[0, 1]`.
    #[inline]
    pub fn identity(self) -> Metric64<C, F> {
        Metric64::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0164::Identity,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Linear normalization: `raw / max`, clamped to `[0, 1]`.
    #[inline]
    pub fn linear(self, max: f64) -> Metric64<C, F> {
        Metric64::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0164::Linear { max },
            _phantom: core::marker::PhantomData,
        }
    }

    /// Increasing sigmoid: `low → ≈0`, `high → ≈1`.
    ///
    /// Uses auto-calibrated steepness `k = 2·ln(1/ε − 1) / (high − low)` where
    /// `ε = 10·f64::EPSILON`. At `raw = low` output ≈ ε, at `raw = high` ≈ 1−ε.
    #[inline]
    pub fn inc_sigmoid(self, low: f64, high: f64) -> Metric64<C, F> {
        Metric64::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0164::IncSigmoid { low, high },
            _phantom: core::marker::PhantomData,
        }
    }

    /// Decreasing sigmoid: `low → ≈1`, `high → ≈0`.
    ///
    /// Same auto-calibrated steepness as [`inc_sigmoid`](Self::inc_sigmoid),
    /// with the sign flipped.
    #[inline]
    pub fn dec_sigmoid(self, low: f64, high: f64) -> Metric64<C, F> {
        Metric64::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0164::DecSigmoid { low, high },
            _phantom: core::marker::PhantomData,
        }
    }

    /// Asymmetric Cauchy (Lorentzian) normalization.
    ///
    /// Peaks at `center` with value 1. `half_left` controls the spread for
    /// `raw < center`, `half_right` for `raw >= center`. When both are equal
    /// this is the classic symmetric Cauchy.
    #[inline]
    pub fn cauchy(self, center: f64, half_left: f64, half_right: f64) -> Metric64<C, F> {
        Metric64::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0164::Cauchy {
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
    pub fn by(self, map01: fn(f64) -> f64) -> Metric64<C, F> {
        Metric64::<C, F> {
            name: self.name,
            measure: self.measure,
            map01: Map0164::Custom(map01),
            _phantom: core::marker::PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Breakdown64 — per-metric detail
// ---------------------------------------------------------------------------

/// A single metric's contribution to the total score.
///
/// Returned by the [`.breakdown()`](ScoreSet64::breakdown) method on a [`Scorer64`].
#[derive(Clone, Debug)]
pub struct Breakdown64 {
    /// Metric name.
    pub name: &'static str,
    /// Raw measured value, before [`Map0164`] normalization.
    pub raw: f64,
    /// Normalized score in `[0, 1]`.
    pub score: f64,
    /// Normalized weight (sums to 1 across all metrics).
    pub weight: f64,
    /// `score * weight`.
    pub contribution: f64,
}

// ---------------------------------------------------------------------------
// MetricTuple64 — trait implemented by tuples of Metric64 for batch evaluation
// ---------------------------------------------------------------------------

/// Trait that lets tuples of [`Metric64`]s be iterated and evaluated.
///
/// Implemented for `(Metric64<C, F0>,)`, `(Metric64<C, F0>, Metric64<C, F1>,)`,
/// etc.  Per-arity impls are generated by xtask into `gen_score_set64.rs`.
///
/// This is an internal helper — users interact with [`Scorer64`] and
/// [`ScoreSet64`] instead.
pub trait MetricTuple64<C> {
    /// Compute the weighted sum of all metric scores.
    fn weighted_sum(&self, weights: &[f64], ctx: &C) -> f64;
    /// Collect per-metric [`Breakdown64`] rows.
    fn collect_breakdown(&self, weights: &[f64], ctx: &C) -> alloc::vec::Vec<Breakdown64>;
}

// ---------------------------------------------------------------------------
// ScoreSet64 — public trait for uniform scorer abstraction
// ---------------------------------------------------------------------------

/// Trait for anything that can score a context and produce a breakdown.
///
/// Implemented by [`Scorer64`], enabling `impl ScoreSet64<Ctx>` or
/// `dyn ScoreSet64<Ctx>` for type erasure across different scorer arities.
pub trait ScoreSet64<Ctx> {
    /// Evaluate the weighted sum against a context.
    fn score(&self, ctx: &Ctx) -> f64;
    /// Produce per-metric breakdown rows.
    fn breakdown(&self, ctx: &Ctx) -> alloc::vec::Vec<Breakdown64>;
}

// ---------------------------------------------------------------------------
// Scorer64 — a validated weighted scorer over a flat tuple of metrics
// ---------------------------------------------------------------------------

/// A validated weighted scorer holding a flat tuple of [`Metric64`]s.
///
/// Created via [`ScorerBuilder64::new().add(...).build()`](ScorerBuilder64).
/// Implements [`ScoreSet64`] for evaluating contexts and producing breakdowns.
pub struct Scorer64<C, T: MetricTuple64<C>> {
    metrics: T,
    weights: alloc::vec::Vec<f64>,
    _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C, T: MetricTuple64<C>> Scorer64<C, T> {
    /// Build from a tuple of metrics and raw weights (validated).
    ///
    /// Called by [`ScorerBuilder64::build`].  Public because the builder lives
    /// in the generated file and needs cross-module access.
    #[inline]
    pub fn new(metrics: T, raw_weights: &[f64]) -> Result<Self, &'static str> {
        for &w in raw_weights {
            let _ = GtZero::witness(w)?;
        }
        let sum: f64 = raw_weights.iter().sum();
        let weights: alloc::vec::Vec<f64> = raw_weights.iter().map(|w| w / sum).collect();
        Ok(Self {
            metrics,
            weights,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<C, T: MetricTuple64<C>> ScoreSet64<C> for Scorer64<C, T> {
    #[inline]
    fn score(&self, ctx: &C) -> f64 {
        self.metrics.weighted_sum(&self.weights, ctx)
    }
    #[inline]
    fn breakdown(&self, ctx: &C) -> alloc::vec::Vec<Breakdown64> {
        self.metrics.collect_breakdown(&self.weights, ctx)
    }
}

// ---------------------------------------------------------------------------
// ScorerBuilder64 — typestate builder for conditional metric composition
// ---------------------------------------------------------------------------

/// Typestate builder for constructing a [`Scorer64`] metric by metric.
///
/// Start with [`ScorerBuilder64::new()`], call [`.add(weight, metric)`](ScorerBuilder64::add)
/// for each metric, and finish with [`.build()`](ScorerBuilder64::build).
///
/// Each `.add()` changes the type of the builder, enabling zero-vtable static
/// dispatch on the resulting scorer.  Conditional composition is done at the
/// `match` branch level (see crate-level docs).
pub struct ScorerBuilder64<C, T> {
    pub(crate) metrics: T,
    pub(crate) weights: alloc::vec::Vec<f64>,
    pub(crate) _phantom: core::marker::PhantomData<fn(&C)>,
}

impl<C> ScorerBuilder64<C, ()> {
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

impl<C> Default for ScorerBuilder64<C, ()> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<C, T: MetricTuple64<C>> ScorerBuilder64<C, T> {
    /// Validate weights and build the [`Scorer64`].
    ///
    /// Normalizes weights so they sum to 1.
    ///
    /// # Errors
    /// Returns `Err` if any weight is not strictly positive (`> 0` and finite).
    #[inline]
    pub fn build(self) -> Result<Scorer64<C, T>, &'static str> {
        Scorer64::new(self.metrics, &self.weights)
    }
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
