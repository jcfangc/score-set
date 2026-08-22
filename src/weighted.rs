use witnessed::{WitnessExt, Witnessed};

use crate::traits::{EvalF32, EvalF64, V01, V01Error, prove_v01_f32, prove_v01_f64};

/// Creates a witnessed `f64` weight in the `[0, 1]` range.
pub fn weight64(value: f64) -> Result<Witnessed<f64, V01>, V01Error> {
    value.witness().by(prove_v01_f64)
}

/// Creates a witnessed `f32` weight in the `[0, 1]` range.
pub fn weight32(value: f32) -> Result<Witnessed<f32, V01>, V01Error> {
    value.witness().by(prove_v01_f32)
}

/// Applies a witnessed `[0, 1]` weight to an `f64` evaluator.
pub struct Weighted64<E> {
    inner: E,
    weight: Witnessed<f64, V01>,
}

impl<E> Weighted64<E> {
    /// Creates a weighted evaluator.
    pub fn new(inner: E, weight: Witnessed<f64, V01>) -> Self {
        Self { inner, weight }
    }

    /// Returns the wrapped evaluator.
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Returns the witnessed weight.
    pub fn weight(&self) -> &Witnessed<f64, V01> {
        &self.weight
    }

    /// Consumes the wrapper and returns the evaluator.
    pub fn into_inner(self) -> E {
        self.inner
    }
}

impl<Ctx, E> EvalF64<Ctx> for Weighted64<E>
where
    Ctx: ?Sized,
    E: EvalF64<Ctx>,
{
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f64 {
        *self.weight * self.inner.eval(ctx)
    }
}

/// Applies a witnessed `[0, 1]` weight to an `f32` evaluator.
pub struct Weighted32<E> {
    inner: E,
    weight: Witnessed<f32, V01>,
}

impl<E> Weighted32<E> {
    /// Creates a weighted evaluator.
    pub fn new(inner: E, weight: Witnessed<f32, V01>) -> Self {
        Self { inner, weight }
    }

    /// Returns the wrapped evaluator.
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Returns the witnessed weight.
    pub fn weight(&self) -> &Witnessed<f32, V01> {
        &self.weight
    }

    /// Consumes the wrapper and returns the evaluator.
    pub fn into_inner(self) -> E {
        self.inner
    }
}

impl<Ctx, E> EvalF32<Ctx> for Weighted32<E>
where
    Ctx: ?Sized,
    E: EvalF32<Ctx>,
{
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f32 {
        *self.weight * self.inner.eval(ctx)
    }
}
