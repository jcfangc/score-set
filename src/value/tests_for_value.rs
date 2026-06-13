use crate::*;

#[test]
fn value01_valid() {
    assert!(0.0_f64.witness().by(Value01::prove()).is_ok());
    assert!(0.5_f64.witness().by(Value01::prove()).is_ok());
    assert!(1.0_f64.witness().by(Value01::prove()).is_ok());
}

#[test]
fn value01_rejects_out_of_range() {
    assert!((-0.1_f64).witness().by(Value01::prove()).is_err());
    assert!(1.1_f64.witness().by(Value01::prove()).is_err());
    assert!(f64::NAN.witness().by(Value01::prove()).is_err());
    assert!(f64::INFINITY.witness().by(Value01::prove()).is_err());
}

#[test]
fn weight_valid() {
    assert!(0.0_f64.witness().by(Weight::prove()).is_ok());
    assert!(5.0_f64.witness().by(Weight::prove()).is_ok());
}

#[test]
fn weight_rejects_negative() {
    assert!((-1.0_f64).witness().by(Weight::prove()).is_err());
}

#[test]
fn normalized_container_prove() {
    assert!(
        vec![0.2_f64, 0.3, 0.5]
            .witness()
            .by(NormalizedContainer::prove())
            .is_ok()
    );
    assert!(
        vec![0.2_f64, 0.3]
            .witness()
            .by(NormalizedContainer::prove())
            .is_err()
    );
    assert!(
        vec![1.2_f64, -0.2]
            .witness()
            .by(NormalizedContainer::prove())
            .is_err()
    );
}

#[test]
fn contribution_arithmetic() {
    let container = vec![1.0_f64]
        .witness()
        .by(NormalizedContainer::prove())
        .unwrap();
    let v = 0.6_f64.witness().by(Value01::prove()).unwrap();
    let w = NormalizedWeight::from_normalized_container(1.0_f64, &container).unwrap();
    let c = Contribution::new(v, w);
    assert!((c.into_inner() - 0.6).abs() < 1e-10);

    let v2 = 0.4_f64.witness().by(Value01::prove()).unwrap();
    let w2 = NormalizedWeight::from_normalized_container(1.0_f64, &container).unwrap();
    let c2 = Contribution::new(v2, w2);

    let sum = c + c2;
    assert!((sum.into_inner() - 1.0).abs() < 1e-10);
}

#[test]
fn score01_construction() {
    let s = Score01::try_new(0.5_f64).unwrap();
    assert!((s.into_inner() - 0.5).abs() < 1e-10);
    assert!(Score01::try_new(-0.1_f64).is_err());
}

#[test]
fn f32_support() {
    assert!(0.5_f32.witness().by(Value01::prove()).is_ok());
    assert!(3.0_f32.witness().by(Weight::prove()).is_ok());
    let container = vec![0.25_f32, 0.25, 0.5]
        .witness()
        .by(NormalizedContainer::prove())
        .unwrap();
    let nw = NormalizedWeight::from_normalized_container(0.25_f32, &container).unwrap();
    assert!((*nw - 0.25_f32).abs() < 1e-7);
}
