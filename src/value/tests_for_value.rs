use crate::*;

#[test]
fn value01_valid() {
    assert!(0.0_f64.witness().by(|v| Value01::prove(*v)).is_ok());
    assert!(0.5_f64.witness().by(|v| Value01::prove(*v)).is_ok());
    assert!(1.0_f64.witness().by(|v| Value01::prove(*v)).is_ok());
}

#[test]
fn value01_rejects_out_of_range() {
    assert!((-0.1_f64).witness().by(|v| Value01::prove(*v)).is_err());
    assert!(1.1_f64.witness().by(|v| Value01::prove(*v)).is_err());
    assert!(f64::NAN.witness().by(|v| Value01::prove(*v)).is_err());
    assert!(f64::INFINITY.witness().by(|v| Value01::prove(*v)).is_err());
}

#[test]
fn weight_valid() {
    assert!(0.0_f64.witness().by(|v| Weight::prove(*v)).is_ok());
    assert!(5.0_f64.witness().by(|v| Weight::prove(*v)).is_ok());
}

#[test]
fn weight_rejects_negative() {
    assert!((-1.0_f64).witness().by(|v| Weight::prove(*v)).is_err());
}

#[test]
fn normalized_container_prove() {
    assert!(
        vec![0.2_f64, 0.3, 0.5]
            .witness()
            .by(|w| NormalizedContainer::prove(w.iter().copied()))
            .is_ok()
    );
    assert!(
        vec![0.2_f64, 0.3]
            .witness()
            .by(|w| NormalizedContainer::prove(w.iter().copied()))
            .is_err()
    );
    assert!(
        vec![1.2_f64, -0.2]
            .witness()
            .by(|w| NormalizedContainer::prove(w.iter().copied()))
            .is_err()
    );
}

#[test]
fn weighted_value_product() {
    let container = vec![1.0_f64]
        .witness()
        .by(|w| NormalizedContainer::prove(w.iter().copied()))
        .unwrap();
    let v = 0.6_f64.witness().by(|v| Value01::prove(*v)).unwrap();
    let w = 1.0_f64
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    let c = v.into_inner() * w.into_inner();
    assert!((c - 0.6).abs() < 1e-10);
}

#[test]
fn f32_support() {
    assert!(0.5_f32.witness().by(|v| Value01::prove(*v)).is_ok());
    assert!(3.0_f32.witness().by(|v| Weight::prove(*v)).is_ok());
    let container = vec![0.25_f32, 0.25, 0.5]
        .witness()
        .by(|w| NormalizedContainer::prove(w.iter().copied()))
        .unwrap();
    let nw = 0.25_f32
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    assert!((*nw - 0.25_f32).abs() < 1e-7);
}
