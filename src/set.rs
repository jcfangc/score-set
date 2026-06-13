use crate::float::ScoreFloat;
use crate::member::Members;
use crate::value::NormalizedContainer;
use core::marker::PhantomData;
use witnessed::WitnessExt;

// ---------------------------------------------------------------------------
// RawScoreSet — pre-normalization
// ---------------------------------------------------------------------------

/// Holds a tuple of [`RawMember`](crate::RawMember)s, ready for normalization.
///
/// Produced by the [`score_set!`](crate::score_set!) macro. Call
/// [`.normalize()`](RawScoreSet::normalize) to convert raw weights to
/// normalized weights and build a [`ScoreSet`].
pub struct RawScoreSet<RawMembers> {
    pub(crate) raw: RawMembers,
}

impl<RawMembers> RawScoreSet<RawMembers> {
    /// Create a new `RawScoreSet` from a tuple of `RawMember`s.
    #[inline]
    pub fn new(raw: RawMembers) -> Self {
        Self { raw }
    }

    /// Normalize raw weights (divide by sum) and validate the resulting set,
    /// producing a [`ScoreSet`].
    pub fn normalize<T, M>(self) -> Result<ScoreSet<T, M>, &'static str>
    where
        T: ScoreFloat,
        M: Members<T, Raw = RawMembers>,
    {
        let raw_weights = M::extract_raw_weights(&self.raw);
        let sum: T = raw_weights.iter().fold(T::zero(), |a, &b| a + b);
        let normalized: Vec<T> = raw_weights.iter().map(|&w| w / sum).collect();
        let container = normalized
            .witness()
            .by(|w| NormalizedContainer::prove(w.iter().copied()))?;
        Ok(ScoreSet {
            members: M::from_raw_with_weights(self.raw, &container),
            _phantom: PhantomData,
        })
    }
}

// ---------------------------------------------------------------------------
// ScoreSet — post-aggregation, ready for scoring
// ---------------------------------------------------------------------------

/// A static weighted set of scoring operators.
///
/// Holds a flat tuple of [`Member`](crate::Member)s with normalized weights.
/// Call [`.score()`](ScoreSet::score) to enter the scoring stage.
pub struct ScoreSet<T: ScoreFloat, Members> {
    pub(crate) members: Members,
    _phantom: PhantomData<T>,
}

impl<T: ScoreFloat, Members> ScoreSet<T, Members> {
    /// Enter the scoring stage, returning a [`ScoreStage`] bound to this set's
    /// members.
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

/// The scoring stage, created by [`ScoreSet::score`].
///
/// Call [`.by(closure)`](ScoreStage::by) to evaluate the set with an
/// arbitrary composition of its members.
pub struct ScoreStage<'a, T: ScoreFloat, Members> {
    members: &'a Members,
    _phantom: PhantomData<T>,
}

impl<'a, T: ScoreFloat, Members> ScoreStage<'a, T, Members> {
    /// Score the set using a user-provided closure.
    ///
    /// The closure receives a reference to the member tuple and is free to
    /// call each member's metric with whatever input shape it needs, combine
    /// contributions, and return a final result.
    #[inline]
    pub fn by<F, R>(self, f: F) -> R
    where
        F: FnOnce(&Members) -> R,
    {
        f(self.members)
    }
}

#[cfg(test)]
mod tests_for_set;
