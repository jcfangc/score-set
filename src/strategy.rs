use crate::float::ScoreFloat;
use crate::member::Members;
use crate::value::Weight;

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
/// - any raw weight fails [`Weight::validate`](crate::Weight::validate)
/// - the sum of weights is zero
/// - the resulting normalized weights fail
///   [`NormalizedWeight::validate_set`](crate::NormalizedWeight::validate_set)
pub fn weighted_mean<T, M>(raw: M::Raw) -> Result<M, &'static str>
where
    T: ScoreFloat,
    M: Members<T>,
{
    use crate::value::NormalizedWeight;

    let raw_weights = M::extract_raw_weights(&raw);

    // Validate each raw weight.
    for &w in &raw_weights {
        Weight::validate(w)?;
    }

    // Compute sum and check positivity.
    let sum: T = raw_weights.iter().fold(T::zero(), |a, &b| a + b);
    if sum <= T::zero() {
        return Err("weighted_mean: sum of weights must be positive");
    }

    // Normalize: weight_i / sum
    let normalized: Vec<T> = raw_weights.iter().map(|&w| w / sum).collect();

    // Validate the complete set (range + sum-to-1), then build.
    NormalizedWeight::validate_set(&normalized)?;

    // SAFETY: validated by NormalizedWeight::validate_set just above.
    Ok(unsafe { M::from_raw_with_weights(raw, &normalized) })
}

#[cfg(test)]
#[path = "tests_for_strategy.rs"]
mod tests_for_strategy;
