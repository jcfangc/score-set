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
fn normalized_container_witness() {
    assert!(NormalizedContainer::witness(vec![0.2_f64, 0.3, 0.5]).is_ok());
    assert!(NormalizedContainer::witness(vec![0.2_f64, 0.3]).is_err());
    assert!(NormalizedContainer::witness(vec![1.2_f64, -0.2]).is_err());
}

#[test]
fn weighted_value_product() {
    let container = NormalizedContainer::witness(vec![1.0_f64]).unwrap();
    let v = Value01::witness(0.6_f64).unwrap();
    let w = 1.0_f64
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    let c = v.into_inner() * w.into_inner();
    assert!((c - 0.6).abs() < 1e-10);
}

#[test]
fn f32_support() {
    assert!(Value01::witness(0.5_f32).is_ok());
    let container = NormalizedContainer::witness(vec![0.25_f32, 0.25, 0.5]).unwrap();
    let nw = 0.25_f32
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    assert!((*nw - 0.25_f32).abs() < 1e-7);
}
