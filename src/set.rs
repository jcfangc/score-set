use crate::float::ScoreFloat;
use crate::member::Members;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// RawScoreSet — pre-aggregation
// ---------------------------------------------------------------------------

/// Holds a tuple of [`RawMember`](crate::RawMember)s, ready for aggregation.
///
/// Produced by the [`score_set!`](crate::score_set!) macro. Call
/// [`.aggregate(strategy)`](RawScoreSet::aggregate) to normalize weights and
/// build a [`ScoreSet`].
pub struct RawScoreSet<RawMembers> {
    pub(crate) raw: RawMembers,
}

impl<RawMembers> RawScoreSet<RawMembers> {
    /// Create a new `RawScoreSet` from a tuple of `RawMember`s.
    #[inline]
    pub fn new(raw: RawMembers) -> Self {
        Self { raw }
    }

    /// Apply an aggregation strategy to normalize weights and produce a
    /// [`ScoreSet`].
    ///
    /// The strategy receives the raw member tuple and returns a normalized one.
    /// Use [`strategy::weighted_mean`](crate::strategy::weighted_mean) or
    /// pass a custom closure.
    pub fn aggregate<T, M, F>(self, strategy: F) -> Result<ScoreSet<T, M>, &'static str>
    where
        T: ScoreFloat,
        M: Members<T, Raw = RawMembers>,
        F: FnOnce(RawMembers) -> Result<M, &'static str>,
    {
        let members = strategy(self.raw)?;
        Ok(ScoreSet {
            members,
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
