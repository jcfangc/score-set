use crate::float::ScoreFloat;
use crate::member::Members;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// RawMetricSet — pre-aggregation
// ---------------------------------------------------------------------------

/// Holds a tuple of [`RawMember`](crate::RawMember)s, ready for aggregation.
///
/// Produced by the [`score_set!`](crate::score_set!) macro. Call
/// [`.aggregate(strategy)`](RawMetricSet::aggregate) to normalize weights and
/// build a [`MetricSet`].
pub struct RawMetricSet<T: ScoreFloat, RawMembers> {
    pub(crate) raw: RawMembers,
    _phantom: PhantomData<T>,
}

impl<T: ScoreFloat, RawMembers> RawMetricSet<T, RawMembers> {
    /// Create a new `RawMetricSet` from a tuple of `RawMember`s.
    #[inline]
    pub fn new(raw: RawMembers) -> Self {
        Self {
            raw,
            _phantom: PhantomData,
        }
    }

    /// Apply an aggregation strategy to normalize weights and produce a
    /// [`MetricSet`].
    ///
    /// The strategy receives the raw member tuple and returns a normalized one.
    /// Use [`strategy::weighted_mean`](crate::strategy::weighted_mean) or
    /// pass a custom closure.
    pub fn aggregate<M, F>(self, strategy: F) -> Result<MetricSet<T, M>, &'static str>
    where
        M: Members<T, Raw = RawMembers>,
        F: FnOnce(RawMembers) -> Result<M, &'static str>,
    {
        let members = strategy(self.raw)?;
        Ok(MetricSet {
            members,
            _phantom: PhantomData,
        })
    }
}

// ---------------------------------------------------------------------------
// MetricSet — post-aggregation, ready for scoring
// ---------------------------------------------------------------------------

/// A static weighted set of scoring operators.
///
/// Holds a flat tuple of [`Member`](crate::Member)s with normalized weights.
/// Call [`.score()`](MetricSet::score) to enter the scoring stage.
pub struct MetricSet<T: ScoreFloat, Members> {
    pub(crate) members: Members,
    _phantom: PhantomData<T>,
}

impl<T: ScoreFloat, Members> MetricSet<T, Members> {
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

/// The scoring stage, created by [`MetricSet::score`].
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
