// ===========================================================================
// finite_metric! — generate a zero-vtable finite metric enum (Layer 2)
// ===========================================================================

/// Declare a finite enum of metric types with static-dispatch `eval`.
///
/// This macro generates an enum whose variants each wrap a concrete metric
/// type. The generated [`Scorable`](crate::Scorable) implementation uses
/// `match` + static dispatch — zero vtable overhead.
///
/// # Syntax — concrete form
///
/// `float` and `subject` are concrete types of the scoring domain:
///
/// ```ignore
/// finite_metric! {
///     metric     => DnaKind,
///     float      => f64,
///     subject    => DnaContext<'static>,
///     dimensions =>
///         Gc(GcRatio),
///         Len(SeqLen),
/// }
/// ```
///
/// # Syntax — generic form
///
/// `T, I` in angle brackets on the metric name declare type parameters.
/// No separate `float`/`subject` keys needed — the names appear in `<T, I>`:
///
/// ```ignore
/// finite_metric! {
///     pub metric => MetricKind<T, I>,
///     dimensions =>
///         Gc(GcRatio<T, I>),
///         Tm(TmScore<T, I>),
///     }
/// }
/// ```
///
/// # Generated items
///
/// - An enum with the listed variants.
/// - A [`Scorable`](crate::Scorable) implementation with
///   static-dispatch `eval` and `name` methods.
///
/// # Requirements on variant types
///
/// Each variant's inner type must provide:
/// - `fn eval(&self, input: &I) -> Witnessed<T, Value01>`
/// - `fn name(&self) -> &str`
///
/// Both [`Metric`](crate::Metric) and `Box<dyn Scorable<T, I>>` satisfy
/// this contract.
#[macro_export]
macro_rules! finite_metric {
    // ---- concrete: metric => Name, float => f64, subject => DnaContext ----
    (
        $(#[$attr:meta])*
        $vis:vis
        metric => $name:ident,
        float => $T:ty,
        subject => $I:ty,
        dimensions => $($variant:ident($ty:ty)),+ $(,)?
    ) => {
        $(#[$attr])*
        #[allow(clippy::pub_enum_variant_fields)]
        $vis enum $name {
            $($variant($ty),)+
        }

        impl $crate::Scorable<$T, $I> for $name {
            #[inline]
            fn eval(&self, input: &$I) -> $crate::Witnessed<$T, $crate::Value01> {
                match self {
                    $(Self::$variant(m) => m.eval(input)),+
                }
            }

            #[inline]
            fn name(&self) -> &str {
                match self {
                    $(Self::$variant(m) => m.name()),+
                }
            }
        }
    };

    // ---- generic: metric => Name<T, I>, dimensions => ... ----
    (
        $(#[$attr:meta])*
        $vis:vis
        metric => $name:ident<$T:ident, $I:ident>,
        dimensions => $($variant:ident($ty:ty)),+ $(,)?
    ) => {
        $(#[$attr])*
        #[allow(clippy::pub_enum_variant_fields)]
        $vis enum $name<$T: $crate::Float, $I> {
            $($variant($ty)),+
        }

        impl<$T: $crate::Float, $I> $crate::Scorable<$T, $I> for $name<$T, $I> {
            #[inline]
            fn eval(&self, input: &$I) -> $crate::Witnessed<$T, $crate::Value01> {
                match self {
                    $(Self::$variant(m) => m.eval(input)),+
                }
            }

            #[inline]
            fn name(&self) -> &str {
                match self {
                    $(Self::$variant(m) => m.name()),+
                }
            }
        }
    };
}

// ===========================================================================
// FiniteScoreSet — weighted set with finite-enum dispatch (Layer 2)
// ===========================================================================

use crate::breakdown::Breakdown;
use crate::dynamic::Scorable;
use crate::float::Float;
use crate::value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};
use core::marker::PhantomData;
use witnessed::{WitnessExt, Witnessed};

// ---------------------------------------------------------------------------
// FiniteMember — a single weighted metric in a FiniteScoreSet
// ---------------------------------------------------------------------------

/// A member of a [`FiniteScoreSet`]: a normalized weight paired with a metric
/// enum variant.
///
/// See [`Member`](crate::Member) for the Layer-1 equivalent.
pub struct FiniteMember<T: Float, E> {
    /// The normalized weight.
    pub weight: Witnessed<T, NormalizedWeight>,
    /// The metric enum variant.
    pub metric: E,
}

impl<T: Float, E> FiniteMember<T, E> {
    /// Compute the weighted contribution of a metric score.
    ///
    /// `contribute(score) = score × normalized_weight`
    #[inline]
    pub fn contribute(&self, value: Witnessed<T, Value01>) -> T {
        value.into_inner() * self.weight.into_inner()
    }

    /// Return a reference to the metric.
    #[inline]
    pub fn metric(&self) -> &E {
        &self.metric
    }
}

// ---------------------------------------------------------------------------
// FiniteScoreSet — weighted set with enum-based static dispatch (Layer 2)
// ---------------------------------------------------------------------------

/// A weighted set of scoring operators using enum-based static dispatch.
///
/// `FiniteScoreSet` stores a `Vec` of [`FiniteMember`]s, each wrapping a
/// variant of a user-declared metric enum. At evaluation time, the enum's
/// `eval` method dispatches via `match` — zero vtable overhead for all
/// non-`Custom` variants.
///
/// Construct via [`FiniteScoreSet::normalize`], then call
/// [`.score()`](FiniteScoreSet::score).
///
/// # Type parameters
///
/// - `T: Float` — the floating-point type (`f32` or `f64`).
/// - `I` — the input type passed to each metric.
/// - `E: Scorable<T, I>` — the metric enum generated by
///   [`finite_metric!`](crate::finite_metric!).
///
/// # Example
///
/// ```ignore
/// let set = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::normalize(vec![
///     (2.0, TestKind::AlwaysZero(ConstMetric::new("zero", 0.0))),
///     (3.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
/// ])?;
///
/// let total = set.sum(&"input");
/// // total = 0.4 * 0 + 0.6 * 1 = 0.6
/// ```
pub struct FiniteScoreSet<T: Float, I, E> {
    members: Vec<FiniteMember<T, E>>,
    _phantom: PhantomData<I>,
}

impl<T: Float, I, E: Scorable<T, I>> FiniteScoreSet<T, I, E> {
    /// Normalize raw weights and validate the resulting set.
    ///
    /// Each weight must be finite and strictly positive. Weights are normalized
    /// to sum to 1.
    pub fn normalize(entries: Vec<(T, E)>) -> Result<Self, &'static str> {
        if entries.is_empty() {
            return Err("FiniteScoreSet: must have at least one member");
        }

        // Validate all weights are > 0
        for (w, _) in &entries {
            GtZero::witness(*w)?;
        }

        let sum: T = entries.iter().fold(T::zero(), |acc, (w, _)| acc + *w);

        let mut normalized: Vec<T> = entries.iter().map(|(w, _)| *w / sum).collect();

        // Sort a copy for binary search in NormalizedWeight
        let mut sorted = normalized.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let container = NormalizedContainer::witness(sorted)?;

        let members: Vec<FiniteMember<T, E>> = entries
            .into_iter()
            .zip(normalized.drain(..))
            .map(|((_, metric), nw)| {
                let weight = nw
                    .witness()
                    .by(|v| NormalizedWeight::from_normalized_container(*v, &container))?;
                Ok(FiniteMember { weight, metric })
            })
            .collect::<Result<Vec<_>, &'static str>>()?;

        Ok(FiniteScoreSet {
            members,
            _phantom: PhantomData,
        })
    }

    /// Evaluate all metrics against `input` and sum their weighted contributions.
    ///
    /// This is the most common aggregation: each metric is evaluated, multiplied
    /// by its normalized weight, and summed. Zero-allocation convenience.
    ///
    /// For custom aggregation, use [`.score()`](Self::score) instead.
    #[inline]
    pub fn sum(&self, input: &I) -> T {
        self.members
            .iter()
            .fold(T::zero(), |acc, m| acc + m.contribute(m.metric.eval(input)))
    }

    /// Enter the scoring stage, returning a reference to all members.
    ///
    /// Use [`.by()`](FiniteScoreStage::by) on the returned stage to apply a
    /// custom aggregation, or [`.sum()`](Self::sum) for the standard
    /// weighted-sum shortcut.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let total = set.score().by(|members| {
    ///     members.iter().fold(0.0, |acc, m| {
    ///         acc + m.contribute(m.metric().eval(&input))
    ///     })
    /// });
    /// ```
    #[inline]
    pub fn score(&self) -> FiniteScoreStage<'_, T, I, E> {
        FiniteScoreStage {
            members: &self.members,
            _phantom: PhantomData,
        }
    }

    /// Return the number of members in this set.
    #[inline]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Return `true` if the set has no members.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Iterate over the members.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &FiniteMember<T, E>> {
        self.members.iter()
    }

    /// Evaluate all metrics against `input` and return a per-metric breakdown.
    ///
    /// Unlike [`.sum()`](Self::sum) which returns only the aggregate,
    /// `breakdown` returns one [`Breakdown`] row per member with the metric's
    /// name, raw score, normalized weight, and weighted contribution.
    #[inline]
    pub fn breakdown(&self, input: &I) -> Vec<Breakdown<'_, T>> {
        self.members
            .iter()
            .map(|m| {
                let score_witness = m.metric.eval(input);
                let score_val: T = *score_witness;
                Breakdown {
                    name: m.metric.name(),
                    score: score_val,
                    weight: m.weight.into_inner(),
                    contribution: m.contribute(score_witness),
                }
            })
            .collect()
    }

    /// Create a builder for incremental construction of a `FiniteScoreSet`.
    ///
    /// Use this when members are not known up front — push them one by one,
    /// then call [`.build()`](FiniteScoreSetBuilder::build) to finalize.
    #[inline]
    pub fn builder() -> FiniteScoreSetBuilder<T, I, E> {
        FiniteScoreSetBuilder {
            entries: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// FiniteScoreStage — member reference for custom aggregation (Layer 2)
// ---------------------------------------------------------------------------

/// The scoring stage for a [`FiniteScoreSet`], created by
/// [`FiniteScoreSet::score`].
///
/// Holds a reference to the set's members. Call
/// [`.by()`](FiniteScoreStage::by) to apply a custom aggregation over the
/// member slice. For the standard weighted-sum shortcut, use
/// [`FiniteScoreSet::sum`] instead.
///
/// # Examples
///
/// ```ignore
/// // Standard weighted sum via the stage:
/// let total = set.score().by(|members| {
///     members.iter().fold(0.0, |acc, m| {
///         acc + m.contribute(m.metric().eval(&input))
///     })
/// });
///
/// // Custom: use only the worst contribution
/// let worst = set.score().by(|members| {
///     members.iter().map(|m| {
///         m.contribute(m.metric().eval(&input))
///     }).fold(f64::INFINITY, f64::min)
/// });
/// ```
pub struct FiniteScoreStage<'a, T: Float, I, E> {
    members: &'a [FiniteMember<T, E>],
    _phantom: PhantomData<I>,
}

impl<'a, T: Float, I, E: Scorable<T, I>> FiniteScoreStage<'a, T, I, E> {
    /// Apply a custom aggregation to the members.
    ///
    /// The closure receives a `&[FiniteMember<T, E>]` — one entry per member
    /// in insertion order. Each [`FiniteMember`] provides
    /// [`.metric()`](FiniteMember::metric) for evaluation and
    /// [`.contribute()`](FiniteMember::contribute) for weighting. The closure
    /// may return any type `R`.
    #[inline]
    pub fn by<F, R>(self, f: F) -> R
    where
        F: FnOnce(&[FiniteMember<T, E>]) -> R,
    {
        f(self.members)
    }
}

// ---------------------------------------------------------------------------
// FiniteScoreSetBuilder — incremental builder for FiniteScoreSet
// ---------------------------------------------------------------------------

/// Incremental builder for [`FiniteScoreSet`].
///
/// Accumulates raw `(weight, variant)` pairs via [`.push()`](Self::push), then
/// normalizes them into a [`FiniteScoreSet`] via [`.build()`](Self::build).
///
/// Each weight is validated on push (must be finite and > 0). Normalization
/// happens once at build time.
///
/// # Examples
///
/// Chain construction:
///
/// ```ignore
/// let set = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::builder()
///     .push(2.0, TestKind::AlwaysZero(const_metric("zero", 0.0)))?
///     .push(3.0, TestKind::AlwaysOne(const_metric("one", 1.0)))?
///     .build()?;
/// ```
///
/// Conditional construction:
///
/// ```ignore
/// let mut builder = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::builder();
/// builder = builder.push(2.0, baseline_variant)?;
/// if enable_extra {
///     builder = builder.push(1.0, extra_variant)?;
/// }
/// let set = builder.build()?;
/// ```
pub struct FiniteScoreSetBuilder<T: Float, I, E> {
    entries: Vec<(T, E)>,
    _phantom: PhantomData<I>,
}

impl<T: Float, I, E: Scorable<T, I>> FiniteScoreSetBuilder<T, I, E> {
    /// Push a metric enum variant with a raw weight into the builder.
    ///
    /// The weight must be finite and strictly positive. This is validated
    /// immediately (fail-fast). Takes and returns `Self` for chaining.
    ///
    /// For incremental construction, rebind the result:
    ///
    /// ```ignore
    /// let mut builder = FiniteScoreSet::builder();
    /// builder = builder.push(2.0, variant_a)?;
    /// if some_condition {
    ///     builder = builder.push(1.0, variant_b)?;
    /// }
    /// let set = builder.build()?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if `weight` is zero, negative, or not finite.
    #[inline]
    pub fn push(mut self, weight: T, variant: E) -> Result<Self, &'static str> {
        GtZero::witness(weight)?;
        self.entries.push((weight, variant));
        Ok(self)
    }

    /// Consume the builder and produce a [`FiniteScoreSet`] with normalized
    /// weights.
    ///
    /// # Errors
    ///
    /// Returns an error if no members were pushed.
    #[inline]
    pub fn build(self) -> Result<FiniteScoreSet<T, I, E>, &'static str> {
        FiniteScoreSet::normalize(self.entries)
    }
}

#[cfg(test)]
mod tests_for_finite;
