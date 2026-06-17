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
    // 1 member
    ($w1:expr => $m1:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![($w1, $crate::FiniteEnum1::M0($m1))])
    };
    // 2 members
    ($w1:expr => $m1:expr, $w2:expr => $m2:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![
            ($w1, $crate::FiniteEnum2::M0($m1)),
            ($w2, $crate::FiniteEnum2::M1($m2)),
        ])
    };
    // 3 members
    ($w1:expr => $m1:expr, $w2:expr => $m2:expr, $w3:expr => $m3:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![
            ($w1, $crate::FiniteEnum3::M0($m1)),
            ($w2, $crate::FiniteEnum3::M1($m2)),
            ($w3, $crate::FiniteEnum3::M2($m3)),
        ])
    };
    // 4 members
    ($w1:expr => $m1:expr, $w2:expr => $m2:expr, $w3:expr => $m3:expr,
     $w4:expr => $m4:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![
            ($w1, $crate::FiniteEnum4::M0($m1)),
            ($w2, $crate::FiniteEnum4::M1($m2)),
            ($w3, $crate::FiniteEnum4::M2($m3)),
            ($w4, $crate::FiniteEnum4::M3($m4)),
        ])
    };
    // 5 members
    ($w1:expr => $m1:expr, $w2:expr => $m2:expr, $w3:expr => $m3:expr,
     $w4:expr => $m4:expr, $w5:expr => $m5:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![
            ($w1, $crate::FiniteEnum5::M0($m1)),
            ($w2, $crate::FiniteEnum5::M1($m2)),
            ($w3, $crate::FiniteEnum5::M2($m3)),
            ($w4, $crate::FiniteEnum5::M3($m4)),
            ($w5, $crate::FiniteEnum5::M4($m5)),
        ])
    };
    // 6 members
    ($w1:expr => $m1:expr, $w2:expr => $m2:expr, $w3:expr => $m3:expr,
     $w4:expr => $m4:expr, $w5:expr => $m5:expr, $w6:expr => $m6:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![
            ($w1, $crate::FiniteEnum6::M0($m1)),
            ($w2, $crate::FiniteEnum6::M1($m2)),
            ($w3, $crate::FiniteEnum6::M2($m3)),
            ($w4, $crate::FiniteEnum6::M3($m4)),
            ($w5, $crate::FiniteEnum6::M4($m5)),
            ($w6, $crate::FiniteEnum6::M5($m6)),
        ])
    };
    // 7 members
    ($w1:expr => $m1:expr, $w2:expr => $m2:expr, $w3:expr => $m3:expr,
     $w4:expr => $m4:expr, $w5:expr => $m5:expr, $w6:expr => $m6:expr,
     $w7:expr => $m7:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![
            ($w1, $crate::FiniteEnum7::M0($m1)),
            ($w2, $crate::FiniteEnum7::M1($m2)),
            ($w3, $crate::FiniteEnum7::M2($m3)),
            ($w4, $crate::FiniteEnum7::M3($m4)),
            ($w5, $crate::FiniteEnum7::M4($m5)),
            ($w6, $crate::FiniteEnum7::M5($m6)),
            ($w7, $crate::FiniteEnum7::M6($m7)),
        ])
    };
    // 8 members
    ($w1:expr => $m1:expr, $w2:expr => $m2:expr, $w3:expr => $m3:expr,
     $w4:expr => $m4:expr, $w5:expr => $m5:expr, $w6:expr => $m6:expr,
     $w7:expr => $m7:expr, $w8:expr => $m8:expr $(,)?) => {
        $crate::FiniteScoreSet::new(vec![
            ($w1, $crate::FiniteEnum8::M0($m1)),
            ($w2, $crate::FiniteEnum8::M1($m2)),
            ($w3, $crate::FiniteEnum8::M2($m3)),
            ($w4, $crate::FiniteEnum8::M3($m4)),
            ($w5, $crate::FiniteEnum8::M4($m5)),
            ($w6, $crate::FiniteEnum8::M5($m6)),
            ($w7, $crate::FiniteEnum8::M6($m7)),
            ($w8, $crate::FiniteEnum8::M7($m8)),
        ])
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
