use crate::float::Float;
use witnessed::{WitnessExt, Witnessed};

// ---------------------------------------------------------------------------
// Value01 — value in [0, 1], finite
// ---------------------------------------------------------------------------

/// Witness credential for a value validated to be finite and in `[0, 1]`.
///
/// ```ignore
/// let v: Witnessed<f64, Value01> = Value01::witness(0.5)?;
/// ```
pub struct Value01;

impl Value01 {
    /// Validate `v` and return a `Witnessed` credential.
    pub fn witness<T: Float>(v: T) -> Result<Witnessed<T, Self>, &'static str> {
        if !v.is_finite() {
            return Err("Value01: value must be finite");
        }
        if v < T::zero() || v > T::one() {
            return Err("Value01: value must be in [0, 1]");
        }
        v.witness().by(|_| Ok(Value01))
    }
}

// ---------------------------------------------------------------------------
// GtZero — strictly greater than zero, finite
// ---------------------------------------------------------------------------

/// Witness credential for a value validated to be finite and strictly > 0.
pub struct GtZero;

impl GtZero {
    /// Validate `v` and return a `Witnessed` credential.
    pub fn witness<T: Float>(v: T) -> Result<Witnessed<T, Self>, &'static str> {
        if !v.is_finite() {
            return Err("GtZero: value must be finite");
        }
        if v <= T::zero() {
            return Err("GtZero: value must be strictly positive");
        }
        v.witness().by(|_| Ok(GtZero))
    }
}

// ---------------------------------------------------------------------------
// NormalizedWeight — weight credential
// ---------------------------------------------------------------------------

/// Witness credential for a single normalized weight.
///
/// Created via [`NormalizedWeight::from_normalized_container`], which
/// binary-searches a validated sorted container to confirm membership.
///
/// ```ignore
/// let set = NormalizedContainer::witness(weights)?;
/// let w = 0.3_f64.witness().by(
///     |v| NormalizedWeight::from_normalized_container(*v, &set)
/// )?;
/// ```
pub struct NormalizedWeight;

impl NormalizedWeight {
    /// Verify `value` is a member of a validated normalized set.
    ///
    /// Binary-searches the sorted `container`.
    pub fn from_normalized_container<T: Float>(
        value: T,
        container: &Witnessed<Vec<T>, NormalizedContainer>,
    ) -> Result<Self, &'static str> {
        if container
            .binary_search_by(|a| a.partial_cmp(&value).unwrap_or(core::cmp::Ordering::Equal))
            .is_ok()
        {
            Ok(NormalizedWeight)
        } else {
            Err("NormalizedWeight: value not found in validated set")
        }
    }
}

// ---------------------------------------------------------------------------
// NormalizedContainer — validated set of normalized weights
// ---------------------------------------------------------------------------

/// Witness credential for a container validated as a complete set of
/// normalized weights (each in `[0, 1]`, sum to 1).
///
/// ```ignore
/// let set = NormalizedContainer::witness(vec![0.2, 0.3, 0.5])?;
/// ```
pub struct NormalizedContainer;

impl NormalizedContainer {
    /// Validate every value is finite, in `[0, 1]`, and the set sums to 1.
    pub fn witness<T: Float>(weights: Vec<T>) -> Result<Witnessed<Vec<T>, Self>, &'static str> {
        let mut sum = T::zero();
        let len = weights.len();
        for &w in &weights {
            if !w.is_finite() {
                return Err("NormalizedContainer: value must be finite");
            }
            if w < T::zero() || w > T::one() {
                return Err("NormalizedContainer: value must be in [0, 1]");
            }
            sum = sum + w;
        }
        let diff = if sum > T::one() {
            sum - T::one()
        } else {
            T::one() - sum
        };
        let tol = T::from_f64(1e-9) * T::from_f64(len as f64).max(T::one());
        if diff > tol {
            return Err("NormalizedContainer: set must sum to 1");
        }
        weights.witness().by(|_| Ok(NormalizedContainer))
    }
}

#[cfg(test)]
mod tests_for_value;
