use crate::float::ScoreFloat;
use crate::member::Members;

// ---------------------------------------------------------------------------
// weighted_mean — normalize weights so they sum to 1
// ---------------------------------------------------------------------------

/// Weighted-mean aggregation strategy.
///
/// Normalizes raw weights to sum to 1 and validates them.
///
/// # Errors
///
/// Returns an error if:
/// - any raw weight is non-finite or negative
/// - the sum of weights is zero
/// - the resulting normalized weights fail
///   [`NormalizedContainer::validate_set`](crate::NormalizedContainer::validate_set)
pub fn weighted_mean<T, M>(raw: M::Raw) -> Result<M, &'static str>
where
    T: ScoreFloat,
    M: Members<T>,
{
    use crate::value::NormalizedContainer;

    let raw_weights = M::extract_raw_weights(&raw);

    // Validate each raw weight.
    for &w in &raw_weights {
        if !w.is_finite() {
            return Err("Weight: value must be finite");
        }
        if w < T::zero() {
            return Err("Weight: value must be non-negative");
        }
    }

    // Compute sum and check positivity.
    let sum: T = raw_weights.iter().fold(T::zero(), |a, &b| a + b);
    if sum <= T::zero() {
        return Err("weighted_mean: sum of weights must be positive");
    }

    // Normalize: weight_i / sum
    let normalized: Vec<T> = raw_weights.iter().map(|&w| w / sum).collect();

    // Validate the complete set (range + sum-to-1), then build.
    NormalizedContainer::validate_set(&normalized)?;

    // SAFETY: validated by NormalizedContainer::validate_set just above.
    Ok(unsafe { M::from_raw_with_weights(raw, &normalized) })
}

#[cfg(test)]
mod tests_for_strategy;
