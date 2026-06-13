use crate::float::ScoreFloat;
use witnessed::Witnessed;

// ---------------------------------------------------------------------------
// Value01 — value in [0, 1], finite
// ---------------------------------------------------------------------------

/// Witness credential for a value validated to be finite and in `[0, 1]`.
///
/// Use with `witnessed`:
/// ```ignore
/// let v = 0.5_f64.witness().by(|v| Value01::prove(*v))?;
/// ```
pub struct Value01;

impl Value01 {
    /// Validate `v` is finite and in `[0, 1]`, returning the credential.
    pub fn prove<T: ScoreFloat>(v: T) -> Result<Self, &'static str> {
        if !v.is_finite() {
            return Err("Value01: value must be finite");
        }
        if v < T::zero() || v > T::one() {
            return Err("Value01: value must be in [0, 1]");
        }
        Ok(Value01)
    }
}

// ---------------------------------------------------------------------------
// NormalizedWeight — weight credential (type tag only)
// ---------------------------------------------------------------------------

/// Witness credential for a single normalized weight.
///
/// Credentials are created via [`NormalizedWeight::from_normalized_container`]
/// which returns a proving closure that binary-searches a validated,
/// sorted container to verify membership.
///
/// ```ignore
/// let set = weights.witness().by(|w| NormalizedContainer::prove(w.iter().copied()))?;
/// let w = 0.3_f64.witness().by(NormalizedWeight::from_normalized_container(&set))?;
/// ```
pub struct NormalizedWeight;

impl NormalizedWeight {
    /// Binary-search `container` for `value`. Returns the credential
    /// tag if the value is a member of the validated normalized set.
    ///
    /// Use with `witnessed`'s `by()`:
    /// ```ignore
    /// let w = value.witness().by(
    ///     |v| NormalizedWeight::from_normalized_container(*v, &container)
    /// )?;
    /// ```
    pub fn from_normalized_container<T: ScoreFloat>(
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
/// let set = weights.witness().by(
///     |w| NormalizedContainer::prove(w.iter().copied())
/// )?;
/// let w = 0.3_f64.witness().by(
///     |v| NormalizedWeight::from_normalized_container(*v, &set)
/// )?;
/// ```
pub struct NormalizedContainer;

impl NormalizedContainer {
    /// Validate every value is finite, in `[0, 1]`, and the set sums to 1.
    ///
    /// Use with `witnessed`:
    /// ```ignore
    /// let set = weights.witness().by(
    ///     |w| NormalizedContainer::prove(w.iter().copied())
    /// )?;
    /// ```
    pub fn prove<T: ScoreFloat>(
        weights: impl IntoIterator<Item = T>,
    ) -> Result<Self, &'static str> {
        let mut sum = T::zero();
        let mut len = 0_usize;
        for w in weights {
            if !w.is_finite() {
                return Err("NormalizedContainer: value must be finite");
            }
            if w < T::zero() || w > T::one() {
                return Err("NormalizedContainer: value must be in [0, 1]");
            }
            sum = sum + w;
            len += 1;
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
        Ok(NormalizedContainer)
    }
}

#[cfg(test)]
mod tests_for_value;
