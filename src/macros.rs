/// Declare a fixed set of `weight => metric` pairs (Layer 1).
///
/// Each weight is validated > 0 via [`GtZero`](crate::GtZero). The macro
/// normalizes weights and returns a [`FixedScoreSet`](crate::FixedScoreSet)
/// directly.
///
/// # Example
///
/// ```ignore
/// let gc  = metric("gc") .measure().by(...).map01().by(...);
/// let len = metric("len").measure().by(...).map01().by(...);
///
/// let ms = fixed_score_set! {
///     2.0 => gc,
///     3.0 => len,
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

/// Declare a finite set of `weight => metric` pairs (Layer 2).
///
/// Metrics are auto-wrapped in an anonymous zero-vtable enum (no explicit
/// [`finite_metric!`](crate::finite_metric!) declaration needed). For named
/// variants or a `Custom` escape hatch, use
/// [`finite_metric!`](crate::finite_metric!) directly.
///
/// # Example
///
/// ```ignore
/// let gc  = metric("gc") .measure().by(...).map01().by(...);
/// let len = metric("len").measure().by(...).map01().by(...);
///
/// let set = finite_score_set! {
///     2.0 => gc,
///     3.0 => len,
/// }?;
///
/// let total = set.sum(&input);
/// ```
#[macro_export]
macro_rules! finite_score_set {
    // empty
    () => {
        $crate::FiniteScoreSet::new(vec![])
    };
    // 1+ members — tuples into trait dispatch, symmetric with `fixed_score_set!` / `Members`
    ($($weight:expr => $metric:expr),+ $(,)?) => {
        $crate::FiniteScoreSet::new(
            $crate::into_finite_entries(
                ($($metric,)+),
                &[$($weight),+],
            )
        )
    };
}

/// Declare a dynamic set of `weight => metric` pairs (Layer 3).
///
/// Metrics are auto-boxed via [`.boxed()`](crate::Metric::boxed) — no manual
/// boxing needed. Weights are normalized and a
/// [`DynamicScoreSet`](crate::DynamicScoreSet) is returned.
///
/// # Example
///
/// ```ignore
/// let gc  = metric("gc") .measure().by(...).map01().by(...);
/// let len = metric("len").measure().by(...).map01().by(...);
///
/// let set = dynamic_score_set! {
///     2.0 => gc,
///     3.0 => len,
/// }?;
///
/// let total = set.sum(&input);
/// ```
#[macro_export]
macro_rules! dynamic_score_set {
    () => {
        $crate::DynamicScoreSet::new(vec![])
    };
    ($($weight:expr => $metric:expr),+ $(,)?) => {
        $crate::DynamicScoreSet::new(vec![
            $(($weight, $metric.boxed())),+
        ])
    };
}
