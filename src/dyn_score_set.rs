use std::marker::PhantomData;

use crate::traits::{EvalF32, EvalF64};

/// A dynamic `f64` score set that sums a list of evaluators.
pub struct DynScoreSet64<Ctx: ?Sized> {
    metrics: Box<[Box<dyn EvalF64<Ctx> + 'static>]>,
}

impl<Ctx: ?Sized> DynScoreSet64<Ctx> {
    /// Creates a builder for `DynScoreSet64`.
    pub fn builder() -> DynScoreSet64Builder<Ctx> {
        DynScoreSet64Builder::default()
    }

    /// Returns the number of stored metrics.
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    /// Returns `true` when no metrics are stored.
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

/// Builds a `DynScoreSet64`.
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
    /// Appends one evaluator to the score set being built.
    pub fn push<E>(&mut self, eval: E)
    where
        E: EvalF64<Ctx> + 'static,
    {
        self.metrics.push(Box::new(eval));
    }

    /// Appends one evaluator and returns the builder.
    pub fn append<E>(mut self, eval: E) -> Self
    where
        E: EvalF64<Ctx> + 'static,
    {
        self.push(eval);
        self
    }

    /// Finalizes the builder into a `DynScoreSet64`.
    pub fn build(self) -> DynScoreSet64<Ctx> {
        DynScoreSet64 {
            metrics: self.metrics.into_boxed_slice(),
        }
    }
}

/// A dynamic `f32` score set that sums a list of evaluators.
pub struct DynScoreSet32<Ctx: ?Sized> {
    metrics: Box<[Box<dyn EvalF32<Ctx> + 'static>]>,
}

impl<Ctx: ?Sized> DynScoreSet32<Ctx> {
    /// Creates a builder for `DynScoreSet32`.
    pub fn builder() -> DynScoreSet32Builder<Ctx> {
        DynScoreSet32Builder::default()
    }

    /// Returns the number of stored metrics.
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    /// Returns `true` when no metrics are stored.
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

/// Builds a `DynScoreSet32`.
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
    /// Appends one evaluator to the score set being built.
    pub fn push<E>(&mut self, eval: E)
    where
        E: EvalF32<Ctx> + 'static,
    {
        self.metrics.push(Box::new(eval));
    }

    /// Appends one evaluator and returns the builder.
    pub fn append<E>(mut self, eval: E) -> Self
    where
        E: EvalF32<Ctx> + 'static,
    {
        self.push(eval);
        self
    }

    /// Finalizes the builder into a `DynScoreSet32`.
    pub fn build(self) -> DynScoreSet32<Ctx> {
        DynScoreSet32 {
            metrics: self.metrics.into_boxed_slice(),
        }
    }
}
