use crate::float::Float;
use crate::value::NormalizedContainer;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// FixedScoreSet — compile-time fixed weighted set (Layer 1)
// ---------------------------------------------------------------------------

/// A compile-time fixed weighted set of scoring operators with normalized
/// weights.
///
/// Construct via [`fixed_score_set!`](crate::fixed_score_set!). Call
/// [`.score()`](FixedScoreSet::score) to enter the scoring stage.
pub struct FixedScoreSet<T: Float, Members> {
    pub(crate) members: Members,
    _phantom: PhantomData<T>,
}

impl<T: Float, Members> FixedScoreSet<T, Members>
where
    Members: crate::Members<T>,
{
    /// Normalize raw weights and validate the resulting set.
    #[doc(hidden)]
    pub fn normalize(raw: Members::Raw) -> Result<Self, &'static str> {
        let raw_weights = Members::extract_raw_weights(&raw);
        let sum: T = raw_weights.iter().fold(T::zero(), |a, &b| a + b);
        let normalized: Vec<T> = raw_weights.iter().map(|&w| w / sum).collect();

        // Sort a clone for the validated container (required by binary search in
        // NormalizedWeight::from_normalized_container). The unsorted `normalized`
        // slice preserves insertion order for per-member lookup by index.
        let mut sorted = normalized.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let container = NormalizedContainer::witness(sorted)?;

        Ok(FixedScoreSet {
            members: Members::from_raw_with_weights(raw, &normalized, &container),
            _phantom: PhantomData,
        })
    }

    /// Enter the scoring stage.
    #[inline]
    pub fn score(&self) -> ScoreStage<'_, T, Members> {
        ScoreStage {
            members: &self.members,
            _phantom: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// ScoreStage — user-provided scoring closure
// ---------------------------------------------------------------------------

/// The scoring stage, created by [`FixedScoreSet::score`].
///
/// Call [`.by(closure)`](ScoreStage::by) to evaluate the set with an
/// arbitrary composition of its members.
pub struct ScoreStage<'a, T: Float, Members> {
    members: &'a Members,
    _phantom: PhantomData<T>,
}

impl<'a, T: Float, Members> ScoreStage<'a, T, Members> {
    /// Score the set using a user-provided closure.
    #[inline]
    pub fn by<F, R>(self, f: F) -> R
    where
        F: FnOnce(&Members) -> R,
    {
        f(self.members)
    }
}

#[cfg(test)]
mod tests_for_fixed;
