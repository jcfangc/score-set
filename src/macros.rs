/// Declare a set of `weight => metric` pairs.
///
/// Each weight is validated > 0 via [`GtZero`](crate::GtZero). The macro
/// collects entries into a [`RawScoreSet`](crate::RawScoreSet). Call
/// [`.normalize()`](crate::RawScoreSet::normalize) to build a
/// [`ScoreSet`](crate::ScoreSet) with normalized weights.
///
/// # Example
///
/// ```ignore
/// let ms = score_set! {
///     2.0 => gc_metric,
///     3.0 => len_metric,
///     5.0 => spec_metric,
/// }?.normalize()?;
/// ```
#[macro_export]
macro_rules! score_set {
    ($($weight:expr => $metric:expr),+ $(,)?) => {
        (|| -> Result<_, &'static str> {
            $crate::RawScoreSet::new((
                $($crate::raw_member($weight, $metric)?,)+
            ))
            .normalize()
        })()
    };
}
