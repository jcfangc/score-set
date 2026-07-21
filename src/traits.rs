use witnessed::Witnessed;

/// Witness attached to values known to be valid for the normalized `[0, 1]`
/// scoring boundary.
pub struct V01;

/// Measures a context and returns a value chosen by the implementation.
///
/// `Output` is the value consumed by a corresponding `Map01F32` or `Map01F64`
/// implementation.
pub trait Measure<Ctx: ?Sized>: Send + Sync {
    /// The value produced by [`Measure::measure`].
    type Output;

    /// Extracts a measurable value from `ctx`.
    fn measure(&self, ctx: &Ctx) -> Self::Output;
}

/// Maps a measurement into the `[0, 1]` range.
///
/// `Input` is associated with the mapper rather than fixed in the trait, so a
/// mapper can consume any measurement type. The normalized result carries the
/// [`V01`] witness.
pub trait Map01F32: Send + Sync {
    /// The value accepted by [`Map01F32::map`].
    type Input;

    /// Converts `value` into a normalized score.
    fn map(&self, value: Self::Input) -> Witnessed<f32, V01>;
}

/// Maps a measurement into the `[0, 1]` range.
///
/// `Input` is associated with the mapper rather than fixed in the trait, so a
/// mapper can consume any measurement type. The normalized result carries the
/// [`V01`] witness.
pub trait Map01F64: Send + Sync {
    /// The value accepted by [`Map01F64::map`].
    type Input;

    /// Converts `value` into a normalized score.
    fn map(&self, value: Self::Input) -> Witnessed<f64, V01>;
}

/// Evaluates a context into an `f64` score.
pub trait EvalF64<Ctx: ?Sized>: Send + Sync {
    /// Computes a score from `ctx`.
    fn eval(&self, ctx: &Ctx) -> f64;
}

/// Evaluates a context into an `f32` score.
pub trait EvalF32<Ctx: ?Sized>: Send + Sync {
    /// Computes a score from `ctx`.
    fn eval(&self, ctx: &Ctx) -> f32;
}
