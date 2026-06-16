/// Declare a fixed set of `weight => metric` pairs (Layer 1).
///
/// Each weight is validated > 0 via [`GtZero`](crate::GtZero). The macro
/// normalizes weights and returns a [`FixedScoreSet`](crate::FixedScoreSet)
/// directly.
///
/// # Example
///
/// ```ignore
/// let ms = fixed_score_set! {
///     2.0 => gc_metric,
///     3.0 => len_metric,
///     5.0 => spec_metric,
/// }?;
/// ```
#[macro_export]
macro_rules! fixed_score_set {
    ($($weight:expr => $metric:expr),+ $(,)?) => {
        (|| -> Result<_, &'static str> {
            $crate::FixedScoreSet::normalize((
                $($crate::raw_member($weight, $metric)?,)+
            ))
        })()
    };
}

/// Declare a dynamic set of `weight => metric` pairs (Layer 3).
///
/// Each metric must be a `Box<dyn DynMetric<T, I>>` (via
/// [`.boxed()`](crate::Metric::boxed) or `Box::new`). Weights are normalized
/// and a [`DynamicScoreSet`](crate::DynamicScoreSet) is returned. The types
/// `T` and `I` are inferred from the boxed metrics.
///
/// # Example
///
/// ```ignore
/// let set = dynamic_score_set! {
///     2.0 => gc_metric.boxed(),
///     3.0 => len_metric.boxed(),
/// }?;
///
/// let total = set.score(&input);
/// ```
#[macro_export]
macro_rules! dynamic_score_set {
    () => {
        $crate::DynamicScoreSet::new(vec![])
    };
    ($($weight:expr => $metric:expr),+ $(,)?) => {
        $crate::DynamicScoreSet::new(vec![
            $(($weight, $metric)),+
        ])
    };
}
