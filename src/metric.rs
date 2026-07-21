use crate::traits::{EvalF32, EvalF64, Map01F32, Map01F64, Measure};

/// A weighted `f64` metric built from a measure and a normalization map.
pub struct Metric64<M, G> {
    measure: M,
    map: G,
    weight: f64,
}

impl<M, G> Metric64<M, G> {
    /// Creates a new `Metric64`.
    pub fn new(measure: M, map: G, weight: f64) -> Self {
        Self {
            measure,
            map,
            weight,
        }
    }
}

impl<Ctx, M, G> EvalF64<Ctx> for Metric64<M, G>
where
    Ctx: ?Sized,
    M: Measure<Ctx>,
    G: Map01F64<Input = M::Output>,
{
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f64 {
        self.weight * *self.map.map(self.measure.measure(ctx))
    }
}

/// A weighted `f32` metric built from a measure and a normalization map.
pub struct Metric32<M, G> {
    measure: M,
    map: G,
    weight: f32,
}

impl<M, G> Metric32<M, G> {
    /// Creates a new `Metric32`.
    pub fn new(measure: M, map: G, weight: f32) -> Self {
        Self {
            measure,
            map,
            weight,
        }
    }
}

impl<Ctx, M, G> EvalF32<Ctx> for Metric32<M, G>
where
    Ctx: ?Sized,
    M: Measure<Ctx>,
    G: Map01F32<Input = M::Output>,
{
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f32 {
        self.weight * *self.map.map(self.measure.measure(ctx))
    }
}
