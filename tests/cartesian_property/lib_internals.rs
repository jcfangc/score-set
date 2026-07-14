//! Library internals — domain-agnostic infrastructure.
//!
//! This module provides the core traits ([`Measure`], [`Map01`]) and the
//! [`Metric`] combinator. It is completely independent of any specific domain.

use core::marker::PhantomData;

/// A measure extracts a raw value from a context.
///
/// Implemented by the domain developer for each measurement they want to score.
/// Measures are typically zero-sized types (field accessors), enabling the
/// compiler to construct all N×M Metric instances at compile time with zero
/// runtime cost.
pub(crate) trait Measure<Ctx> {
    /// Extract the raw measurement from `ctx`.
    fn eval(&self, ctx: &Ctx) -> f32;
}

/// A normalization strategy that maps a raw value to `[0, 1]`.
///
/// **Fixed set, not open for extension.** The library author maintains the
/// complete set of valid `Map01` implementations.
pub(crate) trait Map01 {
    /// Normalize a raw value.
    fn map(&self, v: f32) -> f32;
}

// ---------------------------------------------------------------------------
// Builtin Map01 implementations (all zero-sized types)
// ---------------------------------------------------------------------------

/// Identity: pass through unchanged.
pub(crate) struct Identity;
impl Map01 for Identity {
    #[inline]
    fn map(&self, v: f32) -> f32 {
        v
    }
}

/// Positive: clamp to non-negative. Equivalent to `v.max(0.0)`.
pub(crate) struct Positive;
impl Map01 for Positive {
    #[inline]
    fn map(&self, v: f32) -> f32 {
        v.max(0.0)
    }
}

/// Inverse: `1.0 - v`. Maps large values to small and vice versa.
///
/// Not used in the current toy domain but available as a builtin map option.
#[allow(dead_code)]
pub(crate) struct Inverse;
impl Map01 for Inverse {
    #[inline]
    fn map(&self, v: f32) -> f32 {
        1.0 - v
    }
}

/// Clamp: restrict to `[min, max]`.
pub(crate) struct Clamp {
    pub min: f32,
    pub max: f32,
}
impl Map01 for Clamp {
    #[inline]
    fn map(&self, v: f32) -> f32 {
        v.clamp(self.min, self.max)
    }
}

// ---------------------------------------------------------------------------
// Metric combinator
// ---------------------------------------------------------------------------

/// A single compiled metric: a [`Measure`] paired with a [`Map01`].
///
/// Both `measure` and `map` are stored by value. When both are zero-sized
/// types, the entire `Metric` is a ZST — constructing all N×M combinations
/// costs zero bytes and zero runtime overhead.
///
/// There is **no** `Box`, **no** `dyn`, **no** vtable anywhere in this type.
pub(crate) struct Metric<M, P, Ctx> {
    measure: M,
    map: P,
    _ctx: PhantomData<fn(&Ctx)>,
}

impl<M: Measure<Ctx>, P: Map01, Ctx> Metric<M, P, Ctx> {
    /// Create a new metric from a measure and a normalization strategy.
    #[inline]
    pub(crate) fn new(measure: M, map: P) -> Self {
        Self {
            measure,
            map,
            _ctx: PhantomData,
        }
    }

    /// Evaluate this metric against a context.
    ///
    /// Both calls are statically dispatched — the compiler knows the concrete
    /// types of `M` and `P` at every call site and inlines both `eval` and
    /// `map` into the caller.
    #[inline]
    pub(crate) fn score(&self, ctx: &Ctx) -> f32 {
        self.map.map(self.measure.eval(ctx))
    }
}
