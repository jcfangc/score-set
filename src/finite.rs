// ===========================================================================
// finite_metric! — generate a zero-vtable finite metric enum (Layer 2)
// ===========================================================================

/// Declare a finite enum of metric types with static-dispatch `eval`.
///
/// This macro generates an enum whose variants each wrap a concrete metric
/// type. The generated [`DynMetric`](crate::DynMetric) implementation
/// uses `match` + static dispatch — zero vtable overhead for all
/// business-logic variants. A `Custom(Box<dyn DynMetric<T, I>>)` escape
/// hatch is always present for one-off dynamic metrics.
///
/// # Syntax
///
/// **Generic form** — when variant types are themselves generic over `T, I`:
///
/// ```ignore
/// finite_metric! {
///     pub MetricKind<T, I> =>
///         Gc(GcRatio<T, I>),
///         Tm(TmScore<T, I>),
///         Custom(Box<dyn DynMetric<T, I>>),
/// }
/// ```
///
/// **Concrete form** — when variant types have fixed `T, I`.
/// The `Custom` escape hatch is auto-generated:
///
/// ```ignore
/// finite_metric! {
///     pub DnaMetricKind(f64, DnaContext<'static>) =>
///         Gc(GcRatio),
///         Len(SeqLen),
/// }
/// ```
///
/// # Generated items
///
/// - An enum with the listed variants (generic or concrete).
/// - A [`DynMetric`](crate::DynMetric) implementation with
///   static-dispatch `eval` and `name` methods.
///
/// # Requirements on variant types
///
/// Each variant's inner type must provide:
/// - `fn eval(&self, input: &I) -> Witnessed<T, Value01>`
/// - `fn name(&self) -> &str`
///
/// Both [`Metric`](crate::Metric) and `Box<dyn DynMetric<T, I>>` satisfy
/// this contract.
#[macro_export]
macro_rules! finite_metric {
    // ---- generic form: Foo<T, I> (T, I are type parameters) ----
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident<$T:ident, $I:ident> =>
        $($variant:ident($ty:ty)),+ $(,)?
    ) => {
        $(#[$attr])*
        #[allow(clippy::pub_enum_variant_fields)]
        $vis enum $name<$T: $crate::Float, $I> {
            $($variant($ty)),+
        }

        impl<$T: $crate::Float, $I> $crate::DynMetric<$T, $I> for $name<$T, $I> {
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

    // ---- concrete form: Foo(T, I) (T, I are concrete types) ----
    // Custom escape hatch is auto-generated — users only list their
    // business-logic variants.
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident ( $T:ty , $I:ty ) =>
        $($variant:ident($ty:ty)),+ $(,)?
    ) => {
        $(#[$attr])*
        #[allow(clippy::pub_enum_variant_fields)]
        $vis enum $name {
            $($variant($ty),)+
            Custom(Box<dyn $crate::DynMetric<$T, $I>>),
        }

        impl $crate::DynMetric<$T, $I> for $name {
            #[inline]
            fn eval(&self, input: &$I) -> $crate::Witnessed<$T, $crate::Value01> {
                match self {
                    $(Self::$variant(m) => m.eval(input),)+
                    Self::Custom(c) => c.eval(input),
                }
            }

            #[inline]
            fn name(&self) -> &str {
                match self {
                    $(Self::$variant(m) => m.name(),)+
                    Self::Custom(c) => c.name(),
                }
            }
        }
    };
}

// ===========================================================================
// FiniteScoreSet — weighted set with finite-enum dispatch (Layer 2)
// ===========================================================================

use crate::breakdown::Breakdown;
use crate::dynamic::DynMetric;
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
/// Construct via [`FiniteScoreSet::new`], then call
/// [`.score()`](FiniteScoreSet::score).
///
/// # Type parameters
///
/// - `T: Float` — the floating-point type (`f32` or `f64`).
/// - `I` — the input type passed to each metric.
/// - `E: DynMetric<T, I>` — the metric enum generated by
///   [`finite_metric!`](crate::finite_metric!).
///
/// # Example
///
/// ```ignore
/// let set = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![
///     (2.0, TestKind::AlwaysZero(ConstMetric::new("zero", 0.0))),
///     (3.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
/// ])?;
///
/// let total = set.score(&"input");
/// // total = 0.4 * 0 + 0.6 * 1 = 0.6
/// ```
pub struct FiniteScoreSet<T: Float, I, E> {
    members: Vec<FiniteMember<T, E>>,
    _phantom: PhantomData<I>,
}

impl<T: Float, I, E: DynMetric<T, I>> FiniteScoreSet<T, I, E> {
    /// Create a new `FiniteScoreSet` from a list of `(weight, metric_enum)` pairs.
    ///
    /// Each weight must be finite and strictly positive. Weights are normalized
    /// to sum to 1.
    pub fn new(entries: Vec<(T, E)>) -> Result<Self, &'static str> {
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
    /// by its normalized weight, and summed.
    #[inline]
    pub fn score(&self, input: &I) -> T {
        self.members
            .iter()
            .fold(T::zero(), |acc, m| acc + m.contribute(m.metric.eval(input)))
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
    /// Unlike [`.score()`](Self::score) which returns only the aggregate,
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
}

#[cfg(test)]
mod tests_for_finite;
