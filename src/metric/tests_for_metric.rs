use crate::lab::gc_ratio;
use crate::*;

#[test]
fn metric_builder_eval() {
    let m = metric("gc")
        .measure()
        .by(|dna: &&str| gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());

    assert_eq!(m.name(), "gc");

    let result = m.eval(&"ACGT");
    assert!((*result - 0.5).abs() < 1e-10);
}

#[test]
fn metric_with_empty_input() {
    let m = metric("gc-empty")
        .measure()
        .by(|dna: &&str| gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());

    let result = m.eval(&"");
    assert!((*result - 0.0).abs() < 1e-10);
}

#[test]
fn metric_usize_input() {
    let m = metric("len")
        .measure()
        .by(|len: &usize| *len)
        .map01()
        .by(|raw: &usize, _: &usize| {
            Value01::witness((*raw as f64 / 100.0).min(1.0)).unwrap()
        });

    assert_eq!(m.name(), "len");

    let result = m.eval(&50);
    assert!((*result - 0.5).abs() < 1e-10);

    let result2 = m.eval(&200);
    assert!((*result2 - 1.0).abs() < 1e-10);
}

#[test]
fn map01_identity() {
    let m = metric("id")
        .measure()
        .by(|v: &f64| *v)
        .map01()
        .identity();

    assert!((*m.eval(&0.5) - 0.5).abs() < 1e-10);
    assert!((*m.eval(&1.5) - 1.0).abs() < 1e-10);
    assert!((*m.eval(&-0.5) - 0.0).abs() < 1e-10);
}

#[test]
fn map01_linear() {
    let m = metric("lin")
        .measure()
        .by(|v: &f64| *v)
        .map01()
        .linear(200.0);

    assert!((*m.eval(&100.0) - 0.5).abs() < 1e-10);
    assert!((*m.eval(&300.0) - 1.0).abs() < 1e-10);
}

#[test]
fn map01_sigmoid() {
    let m = metric("sig")
        .measure()
        .by(|v: &f64| *v)
        .map01()
        .sigmoid(0.0, 1.0); // standard logistic

    let result = m.eval(&0.0);
    assert!((*result - 0.5).abs() < 1e-6);
}

#[test]
fn map01_cauchy() {
    let m = metric("cauchy")
        .measure()
        .by(|v: &f64| *v)
        .map01()
        .cauchy(1.0, 1.0);

    let result = m.eval(&1.0);
    assert!((*result - 0.5).abs() < 1e-6);
}

#[test]
fn map01_cauchy_asymmetric() {
    let m = metric("cauchy-asym")
        .measure()
        .by(|v: &f64| *v)
        .map01()
        .cauchy(0.5, 2.0);

    assert!((*m.eval(&-0.5) - 0.5).abs() < 1e-6);
    assert!((*m.eval(&2.0) - 0.5).abs() < 1e-6);
}
