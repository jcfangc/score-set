use crate::erased::ErasedMetric;
use crate::float::Float;
use crate::value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};
use core::marker::PhantomData;
use witnessed::{WitnessExt, Witnessed};

// ---------------------------------------------------------------------------
// DynamicMember — a single weighted metric in a DynamicScoreSet
// ---------------------------------------------------------------------------

/// A member of a [`DynamicScoreSet`]: a normalized weight paired with a
/// type-erased metric.
///
/// See [`Member`](crate::Member) for the Layer-1 equivalent and
/// [`EnumMember`](crate::EnumMember) for the Layer-2 equivalent.
pub struct DynamicMember<T: Float, I> {
    /// The normalized weight.
    pub weight: Witnessed<T, NormalizedWeight>,
    /// The type-erased metric.
    pub metric: Box<dyn ErasedMetric<T, I>>,
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
    pub fn metric(&self) -> &dyn ErasedMetric<T, I> {
        &*self.metric
    }
}

// ---------------------------------------------------------------------------
// DynamicScoreSet — fully dynamic scoring set (Layer 3)
// ---------------------------------------------------------------------------

/// A weighted set of scoring operators using dynamic dispatch.
///
/// `DynamicScoreSet` stores a `Vec` of [`DynamicMember`]s, each holding a
/// `Box<dyn ErasedMetric<T, I>>`. Every evaluation call pays vtable overhead,
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
/// let gc: Box<dyn ErasedMetric<f64, &str>> = Box::new(gc_metric);
/// let len: Box<dyn ErasedMetric<f64, &str>> = Box::new(len_metric);
///
/// let set = DynamicScoreSet::<f64, &str>::new(vec![
///     (2.0, gc),
///     (3.0, len),
/// ])?;
///
/// let total = set.score(&"ACGTACGT");
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
    pub fn new(entries: Vec<(T, Box<dyn ErasedMetric<T, I>>)>) -> Result<Self, &'static str> {
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
    pub fn iter(&self) -> impl Iterator<Item = &DynamicMember<T, I>> {
        self.members.iter()
    }
}

#[cfg(test)]
mod tests_for_dynamic_set;
