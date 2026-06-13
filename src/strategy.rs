use crate::float::ScoreFloat;
use crate::member::Members;
use crate::value::NormalizedContainer;
use witnessed::WitnessExt;

// ---------------------------------------------------------------------------
// weighted_mean — normalize weights so they sum to 1
// ---------------------------------------------------------------------------

/// Weighted-mean aggregation strategy.
///
/// Normalizes raw weights to sum to 1, validates the resulting set via
/// [`NormalizedContainer::prove`], then builds the member tuple.
///
/// # Errors
///
/// Returns an error if any raw weight is non-finite or negative, the sum
/// of weights is zero, or the normalized set fails validation.
pub fn weighted_mean<T, M>(raw: M::Raw) -> Result<M, &'static str>
where
    T: ScoreFloat,
    M: Members<T>,
{
    let raw_weights = M::extract_raw_weights(&raw);

    for &w in &raw_weights {
        if !w.is_finite() {
            return Err("Weight: value must be finite");
        }
        if w < T::zero() {
            return Err("Weight: value must be non-negative");
        }
    }

    let sum: T = raw_weights.iter().fold(T::zero(), |a, &b| a + b);
    if sum <= T::zero() {
        return Err("weighted_mean: sum of weights must be positive");
    }

    let normalized: Vec<T> = raw_weights.iter().map(|&w| w / sum).collect();

    let container = normalized.witness().by(NormalizedContainer::prove())?;

    Ok(M::from_raw_with_weights(raw, &container))
}

#[cfg(test)]
mod tests_for_strategy;
