use crate::float::Float;
use crate::value::Value01;
use witnessed::Witnessed;

// ---------------------------------------------------------------------------
// ErasedMetric — type-erased metric trait (Layer 3 foundation)
// ---------------------------------------------------------------------------

/// A type-erased metric that can be evaluated against input `I`.
///
/// This trait enables dynamic dispatch over heterogeneous metric types via
/// `Box<dyn ErasedMetric<T, I>>`. It is the core abstraction behind
/// [`DynamicScoreSet`](crate::DynamicScoreSet).
///
/// Any [`Metric`](crate::Metric) can be converted into a
/// `Box<dyn ErasedMetric<T, I>>` through the blanket implementation.
///
/// # Examples
///
/// ```ignore
/// use score_set::*;
///
/// let gc = metric("gc")
///     .measure().by(|dna: &&str| gc_ratio(dna))
///     .map01().by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());
///
/// let erased: Box<dyn ErasedMetric<f64, &str>> = Box::new(gc);
/// assert_eq!(erased.name(), "gc");
/// let score = erased.eval(&"ACGT");
/// ```
pub trait ErasedMetric<T: Float, I> {
    /// Evaluate this metric against an input, producing a `[0, 1]` score.
    fn eval(&self, input: &I) -> Witnessed<T, Value01>;

    /// Return the metric's name.
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Blanket impl — any Metric can be used as an ErasedMetric
// ---------------------------------------------------------------------------

impl<T, I, Raw, M, F> ErasedMetric<T, I> for crate::Metric<T, I, Raw, M, F>
where
    T: Float,
    M: Fn(&I) -> Raw,
    F: Fn(&Raw, &I) -> Witnessed<T, Value01>,
{
    #[inline]
    fn eval(&self, input: &I) -> Witnessed<T, Value01> {
        crate::Metric::eval(self, input)
    }

    #[inline]
    fn name(&self) -> &str {
        crate::Metric::name(self)
    }
}

// ---------------------------------------------------------------------------
// ErasedMetric impl for Box<dyn ErasedMetric<T, I>> — enables nesting
// ---------------------------------------------------------------------------

impl<T: Float, I> ErasedMetric<T, I> for Box<dyn ErasedMetric<T, I>> {
    #[inline]
    fn eval(&self, input: &I) -> Witnessed<T, Value01> {
        (**self).eval(input)
    }

    #[inline]
    fn name(&self) -> &str {
        (**self).name()
    }
}

#[cfg(test)]
mod tests_for_erased;
