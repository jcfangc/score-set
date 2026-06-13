use crate::*;

#[test]
fn raw_member_construction() {
    let rm = raw_member(2.5_f64, "metric-data");
    assert!((rm.weight - 2.5).abs() < 1e-10);
    assert_eq!(rm.metric, "metric-data");
}

#[test]
fn member_contribute() {
    let v = Value01::witness(0.6_f64).unwrap();
    let w = unsafe { NormalizedWeight::witness_unchecked(0.5_f64) };
    let m = Member {
        weight: w,
        metric: "test",
    };

    let c = m.contribute(v);
    assert!((c.into_inner() - 0.3).abs() < 1e-10);
}

#[test]
fn member_metric_access() {
    let w = unsafe { NormalizedWeight::witness_unchecked(0.5_f64) };
    let m = Member {
        weight: w,
        metric: 42_u32,
    };

    assert_eq!(*m.metric(), 42);
}

#[test]
fn member_contribute_zero() {
    let v = Value01::witness(1.0_f64).unwrap();
    let w = unsafe { NormalizedWeight::witness_unchecked(0.0_f64) };
    let m = Member {
        weight: w,
        metric: (),
    };

    let c = m.contribute(v);
    assert!((c.into_inner() - 0.0).abs() < 1e-10);
}

#[test]
fn member_contribute_one() {
    let v = Value01::witness(1.0_f64).unwrap();
    let w = unsafe { NormalizedWeight::witness_unchecked(1.0_f64) };
    let m = Member {
        weight: w,
        metric: (),
    };

    let c = m.contribute(v);
    assert!((c.into_inner() - 1.0).abs() < 1e-10);
}
