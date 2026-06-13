use crate::float::ScoreFloat;
use crate::member::Members;
use crate::value::NormalizedContainer;
use witnessed::WitnessExt;

// ---------------------------------------------------------------------------
// weighted_mean — normalize weights so they sum to 1
// ---------------------------------------------------------------------------

/// Weighted-mean aggregation strategy.
///
/// Raw weights already carry [`GtZero`] credentials, so only
/// normalization and set validation are needed here.
///
/// # Errors
///
/// Returns an error if the normalized set fails validation.
pub fn weighted_mean<T, M>(raw: M::Raw) -> Result<M, &'static str>
where
    T: ScoreFloat,
    M: Members<T>,
{
    let raw_weights = M::extract_raw_weights(&raw);

    let sum: T = raw_weights.iter().fold(T::zero(), |a, &b| a + b);

    let normalized: Vec<T> = raw_weights.iter().map(|&w| w / sum).collect();

    let container = normalized
        .witness()
        .by(|w| NormalizedContainer::prove(w.iter().copied()))?;

    Ok(M::from_raw_with_weights(raw, &container))
}

#[cfg(test)]
mod tests_for_strategy;
