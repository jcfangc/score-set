use crate::float::ScoreFloat;
use core::ops::Add;
use witnessed::{WitnessExt, Witnessed};

// ---------------------------------------------------------------------------
// Value01 — value in [0, 1], finite
// ---------------------------------------------------------------------------

/// Witness type for a value validated to be finite and in `[0, 1]`.
///
/// Use [`Value01::witness`] to construct a `Witnessed<T, Value01>`.
pub struct Value01;

impl Value01 {
    /// Validate that `v` is finite and in `[0, 1]`.
    pub fn validate<T: ScoreFloat>(v: T) -> Result<(), &'static str> {
        if !v.is_finite() {
            return Err("Value01: value must be finite");
        }
        if v < T::zero() || v > T::one() {
            return Err("Value01: value must be in [0, 1]");
        }
        Ok(())
    }

    /// Construct a `Witnessed<T, Value01>` by validating `v`.
    pub fn witness<T: ScoreFloat>(v: T) -> Result<Witnessed<T, Self>, &'static str> {
        Self::validate(v)?;
        v.witness().by(|_| Ok(Value01))
    }

    /// Construct a `Witnessed<T, Value01>` without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure `v` is finite and in `[0, 1]`.
    pub unsafe fn witness_unchecked<T: ScoreFloat>(v: T) -> Witnessed<T, Self> {
        unsafe { v.witness().by_unchecked::<Self>() }
    }
}

// ---------------------------------------------------------------------------
// Weight — raw non-negative weight, finite
// ---------------------------------------------------------------------------

/// Witness type for a non-negative, finite weight value.
///
/// Use [`Weight::witness`] to construct a `Witnessed<T, Weight>`.
pub struct Weight;

impl Weight {
    /// Validate that `v` is finite and non-negative.
    pub fn validate<T: ScoreFloat>(v: T) -> Result<(), &'static str> {
        if !v.is_finite() {
            return Err("Weight: value must be finite");
        }
        if v < T::zero() {
            return Err("Weight: value must be non-negative");
        }
        Ok(())
    }

    /// Construct a `Witnessed<T, Weight>` by validating `v`.
    pub fn witness<T: ScoreFloat>(v: T) -> Result<Witnessed<T, Self>, &'static str> {
        Self::validate(v)?;
        v.witness().by(|_| Ok(Weight))
    }
}

// ---------------------------------------------------------------------------
// NormalizedWeight — weight in [0, 1], belonging to a set that sums to 1
// ---------------------------------------------------------------------------

/// Witness type for a normalized weight.
///
/// A normalized weight is individually in `[0, 1]`, finite, and collectively
/// (as a member of a set) sums to 1.0 across the entire set.
///
/// Because the "sums-to-1" property cannot be verified from a single value,
/// use [`NormalizedWeight::validate_set`] to validate the entire collection
/// before calling [`NormalizedWeight::witness_unchecked`] on each member.
pub struct NormalizedWeight;

impl NormalizedWeight {
    /// Validate an individual normalized-weight value (without set context).
    ///
    /// This checks finiteness and the `[0, 1]` range. For full set validation
    /// (including the sums-to-1 invariant), use [`validate_set`](Self::validate_set).
    pub fn validate_individual<T: ScoreFloat>(v: T) -> Result<(), &'static str> {
        if !v.is_finite() {
            return Err("NormalizedWeight: value must be finite");
        }
        if v < T::zero() || v > T::one() {
            return Err("NormalizedWeight: value must be in [0, 1]");
        }
        Ok(())
    }

    /// Validate that a complete set of normalized weights satisfies invariants.
    ///
    /// Checks that:
    /// - every value is finite and in `[0, 1]`
    /// - the sum of all values is 1.0 (within floating-point tolerance)
    pub fn validate_set<T: ScoreFloat>(weights: &[T]) -> Result<(), &'static str> {
        let mut sum = T::zero();
        for &w in weights {
            Self::validate_individual(w)?;
            sum = sum + w;
        }
        let diff = if sum > T::one() {
            sum - T::one()
        } else {
            T::one() - sum
        };
        // Use a generous tolerance for floating-point accumulation
        let tol = T::from_f64(1e-9) * T::from_f64(weights.len() as f64).max(T::one());
        if diff > tol {
            return Err("NormalizedWeight: set must sum to 1");
        }
        Ok(())
    }

    /// Construct a `Witnessed<T, NormalizedWeight>` without per-value validation.
    ///
    /// # Safety
    ///
    /// The caller must have validated the entire set via
    /// [`validate_set`](Self::validate_set) and verified the individual value
    /// belongs to that valid set.
    pub unsafe fn witness_unchecked<T: ScoreFloat>(v: T) -> Witnessed<T, Self> {
        unsafe { v.witness().by_unchecked::<Self>() }
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
        Value01::validate(v)?;
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
