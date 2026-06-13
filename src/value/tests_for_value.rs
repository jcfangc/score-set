use crate::*;

#[test]
fn value01_valid() {
    assert!(Value01::witness(0.0_f64).is_ok());
    assert!(Value01::witness(0.5_f64).is_ok());
    assert!(Value01::witness(1.0_f64).is_ok());
}

#[test]
fn value01_rejects_out_of_range() {
    assert!(Value01::witness(-0.1_f64).is_err());
    assert!(Value01::witness(1.1_f64).is_err());
    assert!(Value01::witness(f64::NAN).is_err());
    assert!(Value01::witness(f64::INFINITY).is_err());
}

#[test]
fn weight_valid() {
    assert!(Weight::witness(0.0_f64).is_ok());
    assert!(Weight::witness(5.0_f64).is_ok());
}

#[test]
fn weight_rejects_negative() {
    assert!(Weight::witness(-1.0_f64).is_err());
}

#[test]
fn normalized_weight_validates_set() {
    // Valid set
    assert!(NormalizedWeight::validate_set(&[0.2_f64, 0.3, 0.5]).is_ok());
    // Sum != 1
    assert!(NormalizedWeight::validate_set(&[0.2_f64, 0.3]).is_err());
    // Out of range
    assert!(NormalizedWeight::validate_set(&[1.2_f64, -0.2]).is_err());
}

#[test]
fn contribution_arithmetic() {
    let v = Value01::witness(0.6_f64).unwrap();
    let w = unsafe { NormalizedWeight::witness_unchecked(0.5_f64) };
    let c = Contribution::new(v, w);
    assert!((c.into_inner() - 0.3).abs() < 1e-10);

    let v2 = Value01::witness(0.4_f64).unwrap();
    let w2 = unsafe { NormalizedWeight::witness_unchecked(0.5_f64) };
    let c2 = Contribution::new(v2, w2);

    let sum = c + c2;
    assert!((sum.into_inner() - 0.5).abs() < 1e-10);
}

#[test]
fn score01_construction() {
    let s = Score01::try_new(0.5_f64).unwrap();
    assert!((s.into_inner() - 0.5).abs() < 1e-10);
    assert!(Score01::try_new(-0.1_f64).is_err());
}

#[test]
fn f32_support() {
    assert!(Value01::witness(0.5_f32).is_ok());
    assert!(Weight::witness(3.0_f32).is_ok());
    let nw = unsafe { NormalizedWeight::witness_unchecked(0.25_f32) };
    assert!((*nw - 0.25_f32).abs() < 1e-7);
}
