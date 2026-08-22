use crate::traits::{EvalF32, EvalF64, Map01F32, Map01F64, Measure};

/// An `f64` evaluator that measures a context and normalizes the result.
pub struct NormalizedEval64<M, G> {
    measure: M,
    map: G,
}

impl<M, G> NormalizedEval64<M, G> {
    /// Creates a normalized evaluator from a measurement and a map.
    pub fn new(measure: M, map: G) -> Self {
        Self { measure, map }
    }
}

impl<Ctx, M, G> EvalF64<Ctx> for NormalizedEval64<M, G>
where
    Ctx: ?Sized,
    M: Measure<Ctx>,
    G: Map01F64<Input = M::Output>,
{
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f64 {
        *self.map.map(self.measure.measure(ctx))
    }
}

/// An `f32` evaluator that measures a context and normalizes the result.
pub struct NormalizedEval32<M, G> {
    measure: M,
    map: G,
}

impl<M, G> NormalizedEval32<M, G> {
    /// Creates a normalized evaluator from a measurement and a map.
    pub fn new(measure: M, map: G) -> Self {
        Self { measure, map }
    }
}

impl<Ctx, M, G> EvalF32<Ctx> for NormalizedEval32<M, G>
where
    Ctx: ?Sized,
    M: Measure<Ctx>,
    G: Map01F32<Input = M::Output>,
{
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f32 {
        *self.map.map(self.measure.measure(ctx))
    }
}
