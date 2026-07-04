use alloc::vec::Vec;
use core::ops::{Add, Mul, Sub};
use witnessed::{WitnessExt, Witnessed};

// ---------------------------------------------------------------------------
// Minimal private trait — only what witness validation needs
// ---------------------------------------------------------------------------

/// Minimal trait for types usable as scores (weights, normalized values).
///
/// Implemented for `f32` and `f64`. Only needed as a bound on witness
/// constructors — downstream code uses concrete types directly.
pub trait ScoreOps:
    Copy + PartialOrd + PartialEq + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
    fn is_finite(self) -> bool;
    fn from_f64(v: f64) -> Self;
}

impl ScoreOps for f32 {
    #[inline]
    fn is_finite(self) -> bool {
        f32::is_finite(self)
    }
    #[inline]
    fn from_f64(v: f64) -> Self {
        v as f32
    }
}

impl ScoreOps for f64 {
    #[inline]
    fn is_finite(self) -> bool {
        f64::is_finite(self)
    }
    #[inline]
    fn from_f64(v: f64) -> Self {
        v
    }
}

// ---------------------------------------------------------------------------
// Value01 — value in [0, 1], finite
// ---------------------------------------------------------------------------

/// Witness credential for a value validated to be finite and in `[0, 1]`.
pub struct Value01;

impl Value01 {
    /// Validate `v` and return a `Witnessed` credential.
    #[inline]
    pub fn witness<T: ScoreOps>(v: T) -> Result<Witnessed<T, Self>, &'static str> {
        if !v.is_finite() {
            return Err("Value01: value must be finite");
        }
        let zero = T::from_f64(0.0);
        let one = T::from_f64(1.0);
        if v < zero || v > one {
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
    #[inline]
    pub fn witness<T: ScoreOps>(v: T) -> Result<Witnessed<T, Self>, &'static str> {
        if !v.is_finite() {
            return Err("GtZero: value must be finite");
        }
        let zero = T::from_f64(0.0);
        if v <= zero {
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
pub struct NormalizedWeight;

impl NormalizedWeight {
    /// Verify `value` is a member of a validated normalized set.
    ///
    /// Binary-searches the sorted `container`.
    #[inline]
    pub fn from_normalized_container<T: ScoreOps>(
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
pub struct NormalizedContainer;

impl NormalizedContainer {
    /// Validate every value is finite, in `[0, 1]`, and the set sums to 1.
    #[inline]
    pub fn witness<T: ScoreOps>(weights: Vec<T>) -> Result<Witnessed<Vec<T>, Self>, &'static str> {
        let zero = T::from_f64(0.0);
        let one = T::from_f64(1.0);
        let mut sum = zero;
        let len = weights.len();
        for &w in &weights {
            if !w.is_finite() {
                return Err("NormalizedContainer: value must be finite");
            }
            if w < zero || w > one {
                return Err("NormalizedContainer: value must be in [0, 1]");
            }
            sum = sum + w;
        }
        let diff = if sum > one { sum - one } else { one - sum };
        let len_f = T::from_f64(len as f64);
        let scale = if len_f > one { len_f } else { one };
        let tol = T::from_f64(1e-9) * scale;
        if diff > tol {
            return Err("NormalizedContainer: set must sum to 1");
        }
        weights.witness().by(|_| Ok(NormalizedContainer))
    }
}

#[cfg(test)]
mod tests_for_value;
