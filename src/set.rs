use crate::float::Float;
use crate::value::NormalizedContainer;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// ScoreSet — normalized weighted set of scoring operators
// ---------------------------------------------------------------------------

/// A static weighted set of scoring operators with normalized weights.
///
/// Construct via [`score_set!`](crate::score_set!) or
/// [`ScoreSet::normalize`]. Call [`.score()`](ScoreSet::score) to enter the
/// scoring stage.
pub struct ScoreSet<T: Float, Members> {
    pub(crate) members: Members,
    _phantom: PhantomData<T>,
}

impl<T: Float, Members> ScoreSet<T, Members>
where
    Members: crate::Members<T>,
{
    /// Normalize raw weights and validate the resulting set.
    pub fn normalize(raw: Members::Raw) -> Result<Self, &'static str> {
        let raw_weights = Members::extract_raw_weights(&raw);
        let sum: T = raw_weights.iter().fold(T::zero(), |a, &b| a + b);
        let normalized: Vec<T> = raw_weights.iter().map(|&w| w / sum).collect();
        let container = NormalizedContainer::witness(normalized)?;
        Ok(ScoreSet {
            members: Members::from_raw_with_weights(raw, &container),
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

/// The scoring stage, created by [`ScoreSet::score`].
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
mod tests_for_set;
