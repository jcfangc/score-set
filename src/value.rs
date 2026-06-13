use crate::float::ScoreFloat;
use core::ops::Add;
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
// Weight — raw non-negative weight, finite
// ---------------------------------------------------------------------------

/// Witness credential for a non-negative, finite weight value.
///
/// Use with `witnessed`:
/// ```ignore
/// let w = 2.0_f64.witness().by(|v| Weight::prove(*v))?;
/// ```
pub struct Weight;

impl Weight {
    /// Validate `v` is finite and non-negative, returning the credential.
    pub fn prove<T: ScoreFloat>(v: T) -> Result<Self, &'static str> {
        if !v.is_finite() {
            return Err("Weight: value must be finite");
        }
        if v < T::zero() {
            return Err("Weight: value must be non-negative");
        }
        Ok(Weight)
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

// ---------------------------------------------------------------------------
// Contribution — Value01 * NormalizedWeight
// ---------------------------------------------------------------------------

/// A single operator's contribution to the final score: `Value01 × NormalizedWeight`.
///
/// Because both operands are in `[0, 1]`, the product is guaranteed to be in `[0, 1]`.
/// This type carries the semantic meaning without redundant runtime checks.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Contribution<T: ScoreFloat>(pub(crate) T);

impl<T: ScoreFloat> Contribution<T> {
    /// Create a contribution from a validated value and its normalized weight.
    #[inline]
    pub fn new(value: Witnessed<T, Value01>, weight: Witnessed<T, NormalizedWeight>) -> Self {
        Self(value.into_inner() * weight.into_inner())
    }

    /// Extract the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// View the inner value.
    #[inline]
    pub fn as_inner(&self) -> T {
        self.0
    }
}

impl<T: ScoreFloat> Add for Contribution<T> {
    type Output = ContributionSum<T>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        ContributionSum(self.0 + rhs.0)
    }
}

impl<T: ScoreFloat> Add<ContributionSum<T>> for Contribution<T> {
    type Output = ContributionSum<T>;

    #[inline]
    fn add(self, rhs: ContributionSum<T>) -> Self::Output {
        ContributionSum(self.0 + rhs.0)
    }
}

// ---------------------------------------------------------------------------
// ContributionSum — sum of Contributions
// ---------------------------------------------------------------------------

/// The accumulated sum of multiple [`Contribution`] values.
///
/// With normalized weights summing to 1, this value should remain in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ContributionSum<T: ScoreFloat>(pub(crate) T);

impl<T: ScoreFloat> ContributionSum<T> {
    /// Start a sum from a single contribution.
    #[inline]
    pub fn from_contribution(c: Contribution<T>) -> Self {
        Self(c.0)
    }

    /// Add another contribution.
    #[inline]
    pub fn add_contribution(self, c: Contribution<T>) -> Self {
        Self(self.0 + c.0)
    }

    /// Extract the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Convert this sum into a final [`Score01`].
    ///
    /// Panics if the value is not in `[0, 1]` (which should not happen
    /// when normalized weights are used correctly).
    #[inline]
    pub fn into_score01(self) -> Score01<T> {
        Score01::from_contribution_sum(self)
    }
}

impl<T: ScoreFloat> Add<Contribution<T>> for ContributionSum<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Contribution<T>) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T: ScoreFloat> Add for ContributionSum<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

// ---------------------------------------------------------------------------
// Score01 — final score in [0, 1]
// ---------------------------------------------------------------------------

/// The final score, guaranteed to be in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Score01<T: ScoreFloat>(pub(crate) T);

impl<T: ScoreFloat> Score01<T> {
    /// Create a `Score01` from a raw value, validating it.
    pub fn try_new(v: T) -> Result<Self, &'static str> {
        if !v.is_finite() {
            return Err("Score01: value must be finite");
        }
        if v < T::zero() || v > T::one() {
            return Err("Score01: value must be in [0, 1]");
        }
        Ok(Self(v))
    }

    /// Create a `Score01` from a contribution sum.
    ///
    /// Panics if the sum is not in `[0, 1]`.
    pub fn from_contribution_sum(sum: ContributionSum<T>) -> Self {
        Self(sum.0)
    }

    /// Extract the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests_for_value;
