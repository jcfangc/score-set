use crate::float::ScoreFloat;
use crate::value::Value01;
use core::marker::PhantomData;
use witnessed::Witnessed;

// ---------------------------------------------------------------------------
// Op builder — score().by() → build()
// ---------------------------------------------------------------------------

/// Entry point: `op("name")`.
pub fn op(name: &'static str) -> OpName {
    OpName { name }
}

/// First stage: a name has been given, waiting for `.score()`.
pub struct OpName {
    name: &'static str,
}

impl OpName {
    /// Enter the score stage.
    pub fn score(self) -> OpScoreStage {
        OpScoreStage { name: self.name }
    }
}

/// Second stage: ready for `.by(score_closure)`.
pub struct OpScoreStage {
    name: &'static str,
}

impl OpScoreStage {
    /// Supply the direct-score closure `Fn(I) -> Witnessed<T, Value01>`.
    ///
    /// Unlike a [`Metric`](crate::Metric), an `Op` directly produces a `[0, 1]`
    /// score from the input without an intermediate raw-value step.
    pub fn by<T: ScoreFloat, I, F>(self, score_fn: F) -> Op<T, I, F> {
        Op {
            name: self.name,
            score_fn,
            _phantom: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Op — the built direct scoring operator
// ---------------------------------------------------------------------------

/// A direct scoring operator: `input → Value01`, without an intermediate
/// raw-value stage.
pub struct Op<T, I, F> {
    pub(crate) name: &'static str,
    pub(crate) score_fn: F,
    _phantom: PhantomData<(T, I)>,
}

impl<T: ScoreFloat, I, F> Op<T, I, F>
where
    F: Fn(I) -> Witnessed<T, Value01>,
{
    /// Evaluate this operator against an input, producing a `[0, 1]` score.
    #[inline]
    pub fn eval(&self, input: I) -> Witnessed<T, Value01> {
        (self.score_fn)(input)
    }

    /// Return the operator's name.
    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }

}

impl<T, I, F: Clone> Clone for Op<T, I, F> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            score_fn: self.score_fn.clone(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests_for_op;
