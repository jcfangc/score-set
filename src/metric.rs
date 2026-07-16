use crate::traits::{Eval, Map01, Measure};

pub struct Metric<M, G> {
    measure: M,
    map: G,
    weight: f64,
}

impl<M, G> Metric<M, G> {
    pub fn new(measure: M, map: G, weight: f64) -> Self {
        Self {
            measure,
            map,
            weight,
        }
    }
}

impl<Ctx, M, G> Eval<Ctx> for Metric<M, G>
where
    Ctx: ?Sized,
    M: Measure<Ctx>,
    G: Map01,
{
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f64 {
        self.weight * self.map.map(self.measure.measure(ctx))
    }
}
