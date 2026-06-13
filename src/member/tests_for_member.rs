use crate::*;

/// Create a validated normalized container with the given sorted weights.
fn normalized_set(weights: Vec<f64>) -> Witnessed<Vec<f64>, NormalizedContainer> {
    NormalizedContainer::witness(weights).unwrap()
}

#[test]
fn raw_member_construction() {
    let rm = raw_member(2.5_f64, "metric-data").unwrap();
    assert!((*rm.weight - 2.5).abs() < 1e-10);
    assert_eq!(rm.metric(), &"metric-data");
}

#[test]
fn member_contribute() {
    let v = Value01::witness(0.6_f64).unwrap();
    let container = normalized_set(vec![1.0]);
    let w = 1.0_f64
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    let m = Member {
        weight: w,
        metric: "test",
    };

    let c = m.contribute(v);
    assert!((c - 0.6).abs() < 1e-10);
}

#[test]
fn member_metric_access() {
    let container = normalized_set(vec![1.0]);
    let w = 1.0_f64
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    let m = Member {
        weight: w,
        metric: 42_u32,
    };

    assert_eq!(*m.metric(), 42);
}

#[test]
fn member_contribute_zero() {
    let v = Value01::witness(1.0_f64).unwrap();
    let container = normalized_set(vec![0.0, 0.5, 0.5]);
    let w = 0.0_f64
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    let m = Member {
        weight: w,
        metric: (),
    };

    let c = m.contribute(v);
    assert!((c - 0.0).abs() < 1e-10);
}

#[test]
fn member_contribute_one() {
    let v = Value01::witness(1.0_f64).unwrap();
    let container = normalized_set(vec![0.5, 0.5]);
    let w = 0.5_f64
        .witness()
        .by(|v| NormalizedWeight::from_normalized_container(*v, &container))
        .unwrap();
    let m = Member {
        weight: w,
        metric: (),
    };

    let c = m.contribute(v);
    assert!((c - 0.5).abs() < 1e-10);
}
