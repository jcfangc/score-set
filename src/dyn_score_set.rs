use std::marker::PhantomData;

use crate::traits::Eval;

pub struct DynScoreSet<Ctx: ?Sized> {
    metrics: Box<[Box<dyn Eval<Ctx> + 'static>]>,
}

impl<Ctx: ?Sized> DynScoreSet<Ctx> {
    pub fn builder() -> DynScoreSetBuilder<Ctx> {
        DynScoreSetBuilder::default()
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }
}

impl<Ctx: ?Sized> Eval<Ctx> for DynScoreSet<Ctx> {
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f64 {
        self.metrics
            .iter()
            .fold(0.0, |sum, metric| sum + metric.eval(ctx))
    }
}

pub struct DynScoreSetBuilder<Ctx: ?Sized> {
    metrics: Vec<Box<dyn Eval<Ctx> + 'static>>,
    marker: PhantomData<fn(&Ctx)>,
}

impl<Ctx: ?Sized> Default for DynScoreSetBuilder<Ctx> {
    fn default() -> Self {
        Self {
            metrics: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Ctx: ?Sized> DynScoreSetBuilder<Ctx> {
    pub fn push<E>(&mut self, eval: E)
    where
        E: Eval<Ctx> + 'static,
    {
        self.metrics.push(Box::new(eval));
    }

    pub fn append<E>(mut self, eval: E) -> Self
    where
        E: Eval<Ctx> + 'static,
    {
        self.push(eval);
        self
    }

    pub fn build(self) -> DynScoreSet<Ctx> {
        DynScoreSet {
            metrics: self.metrics.into_boxed_slice(),
        }
    }
}
