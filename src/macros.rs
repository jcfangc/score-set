/// Declare a set of `weight => metric` pairs.
///
/// Each weight is validated > 0 via [`GtZero`](crate::GtZero). The macro
/// normalizes weights and returns a [`FixedScoreSet`](crate::FixedScoreSet)
/// directly.
///
/// # Example
///
/// ```ignore
/// let ms = score_set! {
///     2.0 => gc_metric,
///     3.0 => len_metric,
///     5.0 => spec_metric,
/// }?;
/// ```
#[macro_export]
macro_rules! score_set {
    ($($weight:expr => $metric:expr),+ $(,)?) => {
        (|| -> Result<_, &'static str> {
            $crate::FixedScoreSet::normalize((
                $($crate::raw_member($weight, $metric)?,)+
            ))
        })()
    };
}
