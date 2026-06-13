use crate::*;

#[test]
fn op_builder_eval() {
    let op = op("thresh")
        .score()
        .by(|v: f64| {
            v.min(1.0)
                .max(0.0)
                .witness()
                .by(|v| Value01::prove(*v))
                .unwrap()
        })
        .build();

    assert_eq!(op.name(), "thresh");
    assert!((*op.eval(0.7) - 0.7).abs() < 1e-10);
    assert!((*op.eval(1.5) - 1.0).abs() < 1e-10);
    assert!((*op.eval(-0.5) - 0.0).abs() < 1e-10);
}

#[test]
fn op_with_f32() {
    let op = op("f32")
        .score()
        .by(|v: f32| {
            v.min(1.0)
                .max(0.0)
                .witness()
                .by(|v| Value01::prove(*v))
                .unwrap()
        })
        .build();

    let result = op.eval(0.5_f32);
    assert!((*result - 0.5_f32).abs() < 1e-7);
}

#[test]
fn op_with_ref_input() {
    let op = op("ref-input")
        .score()
        .by(|s: &str| {
            let v = if s == "yes" { 1.0_f64 } else { 0.0_f64 };
            v.witness().by(|v| Value01::prove(*v)).unwrap()
        })
        .build();

    assert!((*op.eval("yes") - 1.0).abs() < 1e-10);
    assert!((*op.eval("no") - 0.0).abs() < 1e-10);
}
