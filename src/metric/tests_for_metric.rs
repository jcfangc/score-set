use crate::lab::gc_ratio;
use crate::*;

#[test]
fn metric_builder_eval() {
    let m = metric("gc")
        .measure()
        .by(|dna: &str| gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &str| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    assert_eq!(m.name(), "gc");

    let result = m.eval("ACGT"); // gc = 2/4 = 0.5
    assert!((*result - 0.5).abs() < 1e-10);
}

#[test]
fn metric_with_empty_input() {
    let m = metric("gc-empty")
        .measure()
        .by(|dna: &str| gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &str| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let result = m.eval(""); // gc = 0
    assert!((*result - 0.0).abs() < 1e-10);
}

#[test]
fn metric_usize_input() {
    let m = metric("len")
        .measure()
        .by(|len: usize| len)
        .map01()
        .by(|raw: &usize, _: usize| {
            ((*raw as f64 / 100.0).min(1.0))
                .witness()
                .by(|v| Value01::prove(*v))
                .unwrap()
        })
        .build();

    assert_eq!(m.name(), "len");

    let result = m.eval(50); // 50/100 = 0.5
    assert!((*result - 0.5).abs() < 1e-10);

    let result2 = m.eval(200); // capped at 1.0
    assert!((*result2 - 1.0).abs() < 1e-10);
}
