use crate::float::Float;
use crate::value::{GtZero, NormalizedContainer, NormalizedWeight, Value01};
use witnessed::Witnessed;

// ---------------------------------------------------------------------------
// Members trait — maps raw tuple to normalized tuple
// ---------------------------------------------------------------------------

/// Trait that maps raw member tuples to normalized member tuples.
#[doc(hidden)]
pub trait Members<T: Float>: Sized {
    /// The raw (pre-normalization) member tuple.
    type Raw;

    /// Extract raw weight values from the raw member tuple.
    fn extract_raw_weights(raw: &Self::Raw) -> Vec<T>;

    /// Build the normalized member tuple from raw members, the unsorted
    /// normalized-weight slice (insertion order), and a **sorted** validated
    /// normalized container.
    ///
    /// `normalized[idx]` supplies the weight value for member `idx` in insertion
    /// order; the value is then validated against the sorted `container` via
    /// [`NormalizedWeight::from_normalized_container`].
    fn from_raw_with_weights(
        raw: Self::Raw,
        normalized: &[T],
        container: &Witnessed<Vec<T>, NormalizedContainer>,
    ) -> Self;
}

// ---------------------------------------------------------------------------
// RawMember — weight + metric before normalization
// ---------------------------------------------------------------------------

/// A raw member: a strictly-positive weight paired with a metric.
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct RawMember<T: Float, M> {
    pub(crate) weight: Witnessed<T, GtZero>,
    pub(crate) metric: M,
}

impl<T: Float, M> RawMember<T, M> {
    /// Access the raw weight value.
    pub fn weight(&self) -> T {
        *self.weight
    }

    /// Access the metric.
    pub fn metric(&self) -> &M {
        &self.metric
    }
}

/// Construct a `RawMember`, validating that `weight` is strictly positive.
#[doc(hidden)]
#[inline]
pub fn raw_member<T: Float, M>(weight: T, metric: M) -> Result<RawMember<T, M>, &'static str> {
    let w = GtZero::witness(weight)?;
    Ok(RawMember { weight: w, metric })
}

// ---------------------------------------------------------------------------
// Member — normalized weight + metric
// ---------------------------------------------------------------------------

/// A member of a [`MetricSet`](crate::MetricSet): a normalized weight paired
/// with its metric.
#[derive(Debug, Clone, Copy)]
pub struct Member<T: Float, M> {
    /// The normalized weight.
    pub weight: Witnessed<T, NormalizedWeight>,
    /// The metric or operator.
    pub metric: M,
}

impl<T: Float, M> Member<T, M> {
    /// Compute the contribution of a metric score.
    ///
    /// `contribute(score) = score × normalized_weight`
    #[inline]
    pub fn contribute(&self, value: Witnessed<T, Value01>) -> T {
        value.into_inner() * self.weight.into_inner()
    }

    /// Return a reference to the metric.
    #[inline]
    pub fn metric(&self) -> &M {
        &self.metric
    }
}

#[cfg(test)]
mod tests_for_member;
