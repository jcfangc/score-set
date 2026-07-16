/// Measures a context and returns an `f64` value.
pub trait MeasureF64<Ctx: ?Sized>: Send + Sync {
    /// Extracts a measurable value from `ctx`.
    fn measure(&self, ctx: &Ctx) -> f64;
}

/// Maps an `f64` value into the `[0, 1]` range.
pub trait Map01F64: Send + Sync {
    /// Converts `value` into a normalized score.
    fn map(&self, value: f64) -> f64;
}

/// Evaluates a context into an `f64` score.
pub trait EvalF64<Ctx: ?Sized>: Send + Sync {
    /// Computes a score from `ctx`.
    fn eval(&self, ctx: &Ctx) -> f64;
}

/// Measures a context and returns an `f32` value.
pub trait MeasureF32<Ctx: ?Sized>: Send + Sync {
    /// Extracts a measurable value from `ctx`.
    fn measure(&self, ctx: &Ctx) -> f32;
}

/// Maps an `f32` value into the `[0, 1]` range.
pub trait Map01F32: Send + Sync {
    /// Converts `value` into a normalized score.
    fn map(&self, value: f32) -> f32;
}

/// Evaluates a context into an `f32` score.
pub trait EvalF32<Ctx: ?Sized>: Send + Sync {
    /// Computes a score from `ctx`.
    fn eval(&self, ctx: &Ctx) -> f32;
}
