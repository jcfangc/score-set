use crate::float::Float;
use crate::value::Value01;
use witnessed::Witnessed;

// ---------------------------------------------------------------------------
// DynMetric — type-erased metric trait (Layer 3 foundation)
// ---------------------------------------------------------------------------

/// A type-erased metric that can be evaluated against input `I`.
///
/// This trait enables dynamic dispatch over heterogeneous metric types via
/// `Box<dyn DynMetric<T, I>>`. It is the core abstraction behind
/// [`DynamicScoreSet`](crate::DynamicScoreSet).
///
/// Any [`Metric`](crate::Metric) can be converted into a
/// `Box<dyn DynMetric<T, I>>` through the blanket implementation.
///
/// # Examples
///
/// ```ignore
/// use score_set::*;
///
/// let gc = metric("gc")
///     .measure().by(|dna: &&str| gc_ratio(dna))
///     .map01().by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());
///
/// let dyn_metric: Box<dyn DynMetric<f64, &str>> = Box::new(gc);
/// assert_eq!(dyn_metric.name(), "gc");
/// let score = dyn_metric.eval(&"ACGT");
/// ```
pub trait DynMetric<T: Float, I> {
    /// Evaluate this metric against an input, producing a `[0, 1]` score.
    fn eval(&self, input: &I) -> Witnessed<T, Value01>;

    /// Return the metric's name.
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Blanket impl — any Metric is a DynMetric
// ---------------------------------------------------------------------------

impl<T, I, Raw, M, F> DynMetric<T, I> for crate::Metric<T, I, Raw, M, F>
where
    T: Float,
    M: Fn(&I) -> Raw,
    F: Fn(&Raw, &I) -> Witnessed<T, Value01>,
{
    #[inline]
    fn eval(&self, input: &I) -> Witnessed<T, Value01> {
        crate::Metric::eval(self, input)
    }

    #[inline]
    fn name(&self) -> &str {
        crate::Metric::name(self)
    }
}

// ---------------------------------------------------------------------------
// DynMetric impl for Box<dyn DynMetric<T, I>> — enables nesting
// ---------------------------------------------------------------------------

impl<T: Float, I> DynMetric<T, I> for Box<dyn DynMetric<T, I>> {
    #[inline]
    fn eval(&self, input: &I) -> Witnessed<T, Value01> {
        (**self).eval(input)
    }

    #[inline]
    fn name(&self) -> &str {
        (**self).name()
    }
}

// ===========================================================================
// DynamicScoreSet — fully dynamic scoring set (Layer 3)
// ===========================================================================

use crate::breakdown::Breakdown;
use crate::value::{GtZero, NormalizedContainer, NormalizedWeight};
use core::marker::PhantomData;
use witnessed::WitnessExt;

// ---------------------------------------------------------------------------
// DynamicMember — a single weighted metric in a DynamicScoreSet
// ---------------------------------------------------------------------------

/// A member of a [`DynamicScoreSet`]: a normalized weight paired with a
/// type-erased metric.
///
/// See [`Member`](crate::Member) for the Layer-1 equivalent and
/// [`FiniteMember`](crate::FiniteMember) for the Layer-2 equivalent.
pub struct DynamicMember<T: Float, I> {
    /// The normalized weight.
    pub weight: Witnessed<T, NormalizedWeight>,
    /// The type-erased metric.
    pub metric: Box<dyn DynMetric<T, I>>,
}

impl<T: Float, I> DynamicMember<T, I> {
    /// Compute the weighted contribution of a metric score.
    ///
    /// `contribute(score) = score × normalized_weight`
    #[inline]
    pub fn contribute(&self, value: Witnessed<T, Value01>) -> T {
        value.into_inner() * self.weight.into_inner()
    }

    /// Return a reference to the metric.
    #[inline]
    pub fn metric(&self) -> &dyn DynMetric<T, I> {
        &*self.metric
    }
}

// ---------------------------------------------------------------------------
// DynamicScoreSet — fully dynamic scoring set (Layer 3)
// ---------------------------------------------------------------------------

/// A weighted set of scoring operators using dynamic dispatch.
///
/// `DynamicScoreSet` stores a `Vec` of [`DynamicMember`]s, each holding a
/// `Box<dyn DynMetric<T, I>>`. Every evaluation call pays vtable overhead,
/// but the set can contain completely heterogeneous metric types and can be
/// assembled at runtime.
///
/// Construct via [`DynamicScoreSet::new`], then call
/// [`.score()`](DynamicScoreSet::score).
///
/// # Type parameters
///
/// - `T: Float` — the floating-point type (`f32` or `f64`).
/// - `I` — the input type passed to each metric.
///
/// # Example
///
/// ```ignore
/// let gc: Box<dyn DynMetric<f64, &str>> = Box::new(gc_metric);
/// let len: Box<dyn DynMetric<f64, &str>> = Box::new(len_metric);
///
/// let set = DynamicScoreSet::<f64, &str>::new(vec![
///     (2.0, gc),
///     (3.0, len),
/// ])?;
///
/// let total = set.sum(&"ACGTACGT");
/// ```
pub struct DynamicScoreSet<T: Float, I> {
    members: Vec<DynamicMember<T, I>>,
    _phantom: PhantomData<I>,
}

impl<T: Float, I> DynamicScoreSet<T, I> {
    /// Create a new `DynamicScoreSet` from a list of `(weight, metric)` pairs.
    ///
    /// Each weight must be finite and strictly positive. Weights are normalized
    /// to sum to 1.
    pub fn new(entries: Vec<(T, Box<dyn DynMetric<T, I>>)>) -> Result<Self, &'static str> {
        if entries.is_empty() {
            return Err("DynamicScoreSet: must have at least one member");
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

        let members: Vec<DynamicMember<T, I>> = entries
            .into_iter()
            .zip(normalized.drain(..))
            .map(|((_, metric), nw)| {
                let weight = nw
                    .witness()
                    .by(|v| NormalizedWeight::from_normalized_container(*v, &container))?;
                Ok(DynamicMember { weight, metric })
            })
            .collect::<Result<Vec<_>, &'static str>>()?;

        Ok(DynamicScoreSet {
            members,
            _phantom: PhantomData,
        })
    }

    /// Evaluate all metrics against `input` and sum their weighted contributions.
    ///
    /// Zero-allocation convenience for the most common aggregation.
    /// For custom aggregation, use [`.score()`](Self::score) instead.
    #[inline]
    pub fn sum(&self, input: &I) -> T {
        self.members
            .iter()
            .fold(T::zero(), |acc, m| acc + m.contribute(m.metric.eval(input)))
    }

    /// Enter the scoring stage, returning a reference to all members.
    ///
    /// Use [`.by()`](DynamicScoreStage::by) on the returned stage to apply a
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
    pub fn score(&self) -> DynamicScoreStage<'_, T, I> {
        DynamicScoreStage {
            members: &self.members,
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
    pub fn iter(&self) -> impl Iterator<Item = &DynamicMember<T, I>> {
        self.members.iter()
    }

    /// Evaluate all metrics against `input` and return a per-metric breakdown.
    ///
    /// Unlike [`.sum()`](Self::sum) which returns only the aggregate,
    /// `breakdown` returns one [`Breakdown`] row per member with the metric's
    /// name, raw score, normalized weight, and weighted contribution.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for row in set.breakdown(&ctx) {
    ///     println!("{}: {:.3} × {:.3} = {:.3}",
    ///         row.name, row.score, row.weight, row.contribution);
    /// }
    /// ```
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

    /// Create a builder for incremental construction of a `DynamicScoreSet`.
    ///
    /// Use this when members are not known up front — push them one by one,
    /// then call [`.build()`](DynamicScoreSetBuilder::build) to finalize.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let set = DynamicScoreSet::<f64, &str>::builder()
    ///     .push(2.0, gc_metric.boxed())?
    ///     .push(3.0, len_metric.boxed())?
    ///     .build()?;
    /// ```
    #[inline]
    pub fn builder() -> DynamicScoreSetBuilder<T, I> {
        DynamicScoreSetBuilder {
            entries: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// DynamicScoreStage — member reference for custom aggregation (Layer 3)
// ---------------------------------------------------------------------------

/// The scoring stage for a [`DynamicScoreSet`], created by
/// [`DynamicScoreSet::score`].
///
/// Holds a reference to the set's members. Call
/// [`.by()`](DynamicScoreStage::by) to apply a custom aggregation over the
/// member slice. For the standard weighted-sum shortcut, use
/// [`DynamicScoreSet::sum`] instead.
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
/// // Custom: geometric mean of contributions
/// let product = set.score().by(|members| {
///     members.iter().map(|m| {
///         m.contribute(m.metric().eval(&input))
///     }).fold(1.0, |a, c| a * c)
/// });
/// ```
pub struct DynamicScoreStage<'a, T: Float, I> {
    members: &'a [DynamicMember<T, I>],
}

impl<'a, T: Float, I> DynamicScoreStage<'a, T, I> {
    /// Apply a custom aggregation to the members.
    ///
    /// The closure receives a `&[DynamicMember<T, I>]` — one entry per member
    /// in insertion order. Each [`DynamicMember`] provides
    /// [`.metric()`](DynamicMember::metric) for evaluation and
    /// [`.contribute()`](DynamicMember::contribute) for weighting. The closure
    /// may return any type `R`.
    #[inline]
    pub fn by<F, R>(self, f: F) -> R
    where
        F: FnOnce(&[DynamicMember<T, I>]) -> R,
    {
        f(self.members)
    }
}

// ---------------------------------------------------------------------------
// DynamicScoreSetBuilder — incremental builder for DynamicScoreSet
// ---------------------------------------------------------------------------

/// Incremental builder for [`DynamicScoreSet`].
///
/// Accumulates raw `(weight, metric)` pairs via [`.push()`](Self::push), then
/// normalizes them into a [`DynamicScoreSet`] via [`.build()`](Self::build).
///
/// Each weight is validated on push (must be finite and > 0). Normalization
/// happens once at build time.
///
/// # Examples
///
/// Chain construction:
///
/// ```ignore
/// let set = DynamicScoreSet::<f64, &str>::builder()
///     .push(2.0, gc_metric.boxed())?
///     .push(3.0, len_metric.boxed())?
///     .build()?;
/// ```
///
/// Conditional construction:
///
/// ```ignore
/// let mut builder = DynamicScoreSet::<f64, &str>::builder();
/// builder = builder.push(2.0, baseline_metric.boxed())?;
/// if enable_extra {
///     builder = builder.push(1.0, extra_metric.boxed())?;
/// }
/// let set = builder.build()?;
/// ```
pub struct DynamicScoreSetBuilder<T: Float, I> {
    entries: Vec<(T, Box<dyn DynMetric<T, I>>)>,
}

impl<T: Float, I> DynamicScoreSetBuilder<T, I> {
    /// Push a metric with a raw weight into the builder.
    ///
    /// The weight must be finite and strictly positive. This is validated
    /// immediately (fail-fast). Takes and returns `Self` for chaining.
    ///
    /// For incremental construction, rebind the result:
    ///
    /// ```ignore
    /// let mut builder = DynamicScoreSet::builder();
    /// builder = builder.push(2.0, gc_metric.boxed())?;
    /// if some_condition {
    ///     builder = builder.push(1.0, extra_metric.boxed())?;
    /// }
    /// let set = builder.build()?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if `weight` is zero, negative, or not finite.
    #[inline]
    pub fn push(
        mut self,
        weight: T,
        metric: Box<dyn DynMetric<T, I>>,
    ) -> Result<Self, &'static str> {
        GtZero::witness(weight)?;
        self.entries.push((weight, metric));
        Ok(self)
    }

    /// Consume the builder and produce a [`DynamicScoreSet`] with normalized
    /// weights.
    ///
    /// # Errors
    ///
    /// Returns an error if no members were pushed.
    #[inline]
    pub fn build(self) -> Result<DynamicScoreSet<T, I>, &'static str> {
        DynamicScoreSet::new(self.entries)
    }
}

#[cfg(test)]
mod tests_for_dynamic;
