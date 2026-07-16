use std::marker::PhantomData;

use crate::traits::{EvalF32, EvalF64};

pub struct DynScoreSet64<Ctx: ?Sized> {
    metrics: Box<[Box<dyn EvalF64<Ctx> + 'static>]>,
}

impl<Ctx: ?Sized> DynScoreSet64<Ctx> {
    pub fn builder() -> DynScoreSet64Builder<Ctx> {
        DynScoreSet64Builder::default()
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }
}

impl<Ctx: ?Sized> EvalF64<Ctx> for DynScoreSet64<Ctx> {
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f64 {
        self.metrics
            .iter()
            .fold(0.0, |sum, metric| sum + metric.eval(ctx))
    }
}

pub struct DynScoreSet64Builder<Ctx: ?Sized> {
    metrics: Vec<Box<dyn EvalF64<Ctx> + 'static>>,
    marker: PhantomData<fn(&Ctx)>,
}

impl<Ctx: ?Sized> Default for DynScoreSet64Builder<Ctx> {
    fn default() -> Self {
        Self {
            metrics: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Ctx: ?Sized> DynScoreSet64Builder<Ctx> {
    pub fn push<E>(&mut self, eval: E)
    where
        E: EvalF64<Ctx> + 'static,
    {
        self.metrics.push(Box::new(eval));
    }

    pub fn append<E>(mut self, eval: E) -> Self
    where
        E: EvalF64<Ctx> + 'static,
    {
        self.push(eval);
        self
    }

    pub fn build(self) -> DynScoreSet64<Ctx> {
        DynScoreSet64 {
            metrics: self.metrics.into_boxed_slice(),
        }
    }
}

pub struct DynScoreSet32<Ctx: ?Sized> {
    metrics: Box<[Box<dyn EvalF32<Ctx> + 'static>]>,
}

impl<Ctx: ?Sized> DynScoreSet32<Ctx> {
    pub fn builder() -> DynScoreSet32Builder<Ctx> {
        DynScoreSet32Builder::default()
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }
}

impl<Ctx: ?Sized> EvalF32<Ctx> for DynScoreSet32<Ctx> {
    #[inline]
    fn eval(&self, ctx: &Ctx) -> f32 {
        self.metrics
            .iter()
            .fold(0.0, |sum, metric| sum + metric.eval(ctx))
    }
}

pub struct DynScoreSet32Builder<Ctx: ?Sized> {
    metrics: Vec<Box<dyn EvalF32<Ctx> + 'static>>,
    marker: PhantomData<fn(&Ctx)>,
}

impl<Ctx: ?Sized> Default for DynScoreSet32Builder<Ctx> {
    fn default() -> Self {
        Self {
            metrics: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Ctx: ?Sized> DynScoreSet32Builder<Ctx> {
    pub fn push<E>(&mut self, eval: E)
    where
        E: EvalF32<Ctx> + 'static,
    {
        self.metrics.push(Box::new(eval));
    }

    pub fn append<E>(mut self, eval: E) -> Self
    where
        E: EvalF32<Ctx> + 'static,
    {
        self.push(eval);
        self
    }

    pub fn build(self) -> DynScoreSet32<Ctx> {
        DynScoreSet32 {
            metrics: self.metrics.into_boxed_slice(),
        }
    }
}
