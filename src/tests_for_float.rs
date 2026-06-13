use crate::ScoreFloat;
use crate::float::sealed::Float;

#[test]
fn f64_is_score_float() {
    fn assert_score_float<T: ScoreFloat>() {}
    assert_score_float::<f64>();
}

#[test]
fn f32_is_score_float() {
    fn assert_score_float<T: ScoreFloat>() {}
    assert_score_float::<f32>();
}

#[test]
fn zero_and_one() {
    assert_eq!(f64::zero(), 0.0);
    assert_eq!(f64::one(), 1.0);
    assert_eq!(f32::zero(), 0.0_f32);
    assert_eq!(f32::one(), 1.0_f32);
}

#[test]
fn finite_check() {
    assert!(f64::is_finite(1.0));
    assert!(!f64::is_finite(f64::NAN));
    assert!(!f64::is_finite(f64::INFINITY));
    assert!(f32::is_finite(1.0_f32));
    assert!(!f32::is_finite(f32::NAN));
}

#[test]
fn from_f64_roundtrip() {
    assert_eq!(f64::from_f64(0.5).into_f64(), 0.5);
    assert!((f32::from_f64(0.5).into_f64() - 0.5).abs() < 1e-6);
}

#[test]
fn abs_and_min_max() {
    assert_eq!(f64::abs(-1.0), 1.0);
    assert_eq!(f64::min(2.0, 3.0), 2.0);
    assert_eq!(f64::max(2.0, 3.0), 3.0);
}
