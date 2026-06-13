use crate::float::ScoreFloat;
use crate::value::Value01;
use core::marker::PhantomData;
use witnessed::Witnessed;

// ---------------------------------------------------------------------------
// Metric builder — measure().by() → map01().by() → build()
// ---------------------------------------------------------------------------

/// Entry point: `metric("name")`.
pub fn metric(name: &'static str) -> MetricName {
    MetricName { name }
}

/// First stage: a name has been given, waiting for `.measure()`.
pub struct MetricName {
    name: &'static str,
}

impl MetricName {
    /// Enter the measure stage.
    pub fn measure(self) -> MeasureStage {
        MeasureStage { name: self.name }
    }
}

/// Second stage: ready for `.by(measure_closure)`.
pub struct MeasureStage {
    name: &'static str,
}

impl MeasureStage {
    /// Supply the measure closure `Fn(I) -> Raw`.
    pub fn by<I, Raw, M>(self, measure: M) -> MeasureSet<I, Raw, M> {
        MeasureSet {
            name: self.name,
            measure,
            _phantom: PhantomData,
        }
    }
}

/// Third stage: measure closure is set, waiting for `.map01()`.
pub struct MeasureSet<I, Raw, M> {
    name: &'static str,
    measure: M,
    _phantom: PhantomData<(I, Raw)>,
}

impl<I, Raw, M> MeasureSet<I, Raw, M> {
    /// Enter the map01 stage.
    pub fn map01(self) -> Map01Stage<I, Raw, M> {
        Map01Stage {
            name: self.name,
            measure: self.measure,
            _phantom: PhantomData,
        }
    }
}

/// Fourth stage: ready for `.by(map01_closure)`.
pub struct Map01Stage<I, Raw, M> {
    name: &'static str,
    measure: M,
    _phantom: PhantomData<(I, Raw)>,
}

impl<I, Raw, M> Map01Stage<I, Raw, M> {
    /// Supply the map01 closure `Fn(&Raw, I) -> Witnessed<T, Value01>`.
    ///
    /// The type `T: ScoreFloat` is inferred from the return type of the closure.
    pub fn by<T: ScoreFloat, F>(self, map01: F) -> Metric<T, I, Raw, M, F> {
        Metric {
            name: self.name,
            measure: self.measure,
            map01,
            _phantom: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Metric — the built scoring operator
// ---------------------------------------------------------------------------

/// A scoring operator built via the `measure().by() → map01().by()` pipeline.
///
/// Stores two closures:
/// - `measure: Fn(I) -> Raw` — maps input to an intermediate raw value.
/// - `map01: Fn(&Raw, I) -> Witnessed<T, Value01>` — maps the raw value back
///   alongside the original input to a validated `[0, 1]` score.
///
/// `eval(input)` composes the two closures.
pub struct Metric<T, I, Raw, M, F> {
    pub(crate) name: &'static str,
    pub(crate) measure: M,
    pub(crate) map01: F,
    _phantom: PhantomData<(T, I, Raw)>,
}

impl<T: ScoreFloat, I, Raw, M, F> Metric<T, I, Raw, M, F>
where
    I: Copy,
    M: Fn(I) -> Raw,
    F: Fn(&Raw, I) -> Witnessed<T, Value01>,
{
    /// Evaluate this metric against an input, producing a `[0, 1]` score.
    #[inline]
    pub fn eval(&self, input: I) -> Witnessed<T, Value01> {
        let raw = (self.measure)(input);
        (self.map01)(&raw, input)
    }

    /// Return the metric's name.
    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }
}

// Make Metric clone-able when closures are clone-able.
impl<T, I, Raw, M: Clone, F: Clone> Clone for Metric<T, I, Raw, M, F> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            measure: self.measure.clone(),
            map01: self.map01.clone(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests_for_metric;
