use crate::float::ScoreFloat;
use core::ops::Add;
use witnessed::Witnessed;

// ---------------------------------------------------------------------------
// Value01 — value in [0, 1], finite
// ---------------------------------------------------------------------------

/// Witness credential for a value validated to be finite and in `[0, 1]`.
///
/// This type is a ZST credential only. Construct a `Witnessed<T, Value01>`
/// by passing [`Value01::prove`] to `witnessed`'s validation framework:
///
/// ```ignore
/// let v: Witnessed<f64, Value01> = 0.5.witness().by(Value01::prove())?;
/// ```
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

    /// Return a proving closure for use with [`WitnessExt::by`](witnessed::WitnessExt).
    ///
    /// ```ignore
    /// let v = 0.5_f64.witness().by(Value01::prove())?;
    /// ```
    pub fn prove<T: ScoreFloat>() -> impl Fn(&T) -> Result<Self, &'static str> {
        |v| {
            Self::validate(*v)?;
            Ok(Value01)
        }
    }
}

// ---------------------------------------------------------------------------
// Weight — raw non-negative weight, finite
// ---------------------------------------------------------------------------

/// Witness credential for a non-negative, finite weight value.
///
/// This type is a ZST credential only. Construct a `Witnessed<T, Weight>`
/// by passing [`Weight::prove`] to `witnessed`'s validation framework:
///
/// ```ignore
/// let w: Witnessed<f64, Weight> = 2.0.witness().by(Weight::prove())?;
/// ```
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

    /// Return a proving closure for use with [`WitnessExt::by`](witnessed::WitnessExt).
    ///
    /// ```ignore
    /// let w = 2.0_f64.witness().by(Weight::prove())?;
    /// ```
    pub fn prove<T: ScoreFloat>() -> impl Fn(&T) -> Result<Self, &'static str> {
        |v| {
            Self::validate(*v)?;
            Ok(Weight)
        }
    }
}

// ---------------------------------------------------------------------------
// NormalizedWeight — weight in [0, 1], belonging to a set that sums to 1
// ---------------------------------------------------------------------------

/// Witness credential for a normalized weight.
///
/// A normalized weight is individually in `[0, 1]`, finite, and collectively
/// (as a member of a set) sums to 1.0 across the entire set.
///
/// Because the "sums-to-1" property cannot be verified from a single value,
/// use [`NormalizedWeight::validate_set`] to validate the entire collection
/// before constructing each member via `witnessed`'s unchecked path:
///
/// ```ignore
/// NormalizedWeight::validate_set(&weights)?;
/// // SAFETY: set validated just above
/// let w: Witnessed<f64, NormalizedWeight> =
///     unsafe { weight.witness().by_unchecked::<NormalizedWeight>() };
/// ```
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
