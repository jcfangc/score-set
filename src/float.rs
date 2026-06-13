use core::fmt::Debug;
use core::ops::{Add, Div, Mul, Sub};

/// Sealed trait for floating-point types supported by this crate.
///
/// Implemented for `f32` and `f64` only — downstream crates cannot add new impls.
pub(crate) mod sealed {
    use super::*;

    pub trait SealedFloat:
        Copy
        + PartialOrd
        + PartialEq
        + Debug
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + 'static
    {
        fn zero() -> Self;
        fn one() -> Self;
        fn is_finite(self) -> bool;
        fn from_f64(v: f64) -> Self;
        fn abs(self) -> Self;
        fn min(self, other: Self) -> Self;
        fn max(self, other: Self) -> Self;
    }

    impl SealedFloat for f32 {
        #[inline]
        fn zero() -> Self {
            0.0
        }
        #[inline]
        fn one() -> Self {
            1.0
        }
        #[inline]
        fn is_finite(self) -> bool {
            f32::is_finite(self)
        }
        #[inline]
        fn from_f64(v: f64) -> Self {
            v as f32
        }
        #[inline]
        fn abs(self) -> Self {
            f32::abs(self)
        }
        #[inline]
        fn min(self, other: Self) -> Self {
            f32::min(self, other)
        }
        #[inline]
        fn max(self, other: Self) -> Self {
            f32::max(self, other)
        }
    }

    impl SealedFloat for f64 {
        #[inline]
        fn zero() -> Self {
            0.0
        }
        #[inline]
        fn one() -> Self {
            1.0
        }
        #[inline]
        fn is_finite(self) -> bool {
            f64::is_finite(self)
        }
        #[inline]
        fn from_f64(v: f64) -> Self {
            v
        }
        #[inline]
        fn abs(self) -> Self {
            f64::abs(self)
        }
        #[inline]
        fn min(self, other: Self) -> Self {
            f64::min(self, other)
        }
        #[inline]
        fn max(self, other: Self) -> Self {
            f64::max(self, other)
        }
    }
}

/// Public trait bound for floating-point types used throughout `score-set`.
///
/// Blanket-implemented for `f32` and `f64`. Sealed so downstream cannot add new impls.
pub trait Float: sealed::SealedFloat {}

impl<T: sealed::SealedFloat> Float for T {}

#[cfg(test)]
mod tests_for_float;
