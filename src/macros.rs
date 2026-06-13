/// Declare a set of `weight => metric` pairs.
///
/// The macro only wraps each entry into a [`RawMember`](crate::RawMember) and
/// collects them into a [`RawScoreSet`](crate::RawScoreSet). Call
/// `.aggregate(strategy)` on the result to normalize weights and build a
/// [`ScoreSet`](crate::ScoreSet).
///
/// # Example
///
/// ```ignore
/// let ms = score_set! {
///     2.0 => gc_metric,
///     3.0 => len_metric,
///     5.0 => spec_metric,
/// }.aggregate(score_set::strategy::weighted_mean)?;
/// ```
#[macro_export]
macro_rules! score_set {
    ($($weight:expr => $metric:expr),+ $(,)?) => {
        (|| -> Result<_, &'static str> {
            Ok($crate::RawScoreSet::new((
                $($crate::raw_member($weight, $metric)?,)+
            )))
        })()
    };
}
