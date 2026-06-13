use crate::lab::gc_ratio;
use crate::*;

#[test]
fn readme_example() -> Result<(), &'static str> {
    let gc = metric("gc")
        .measure()
        .by(|dna: &str| gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &str| Value01::witness((*raw)).unwrap());

    let len = metric("len")
        .measure()
        .by(|len: usize| len)
        .map01()
        .by(|raw: &usize, _: usize| Value01::witness((*raw as f64 / 100.0).min(1.0)).unwrap());

    let ms = score_set! {
        2.0 => gc,
        3.0 => len,
    }?;

    let dna = "ACGTACGT";

    let score = ms.score().by(|(gc, len)| {
        gc.contribute(gc.metric().eval(dna)) + len.contribute(len.metric().eval(dna.len()))
    });

    let inner = score;
    assert!(inner >= 0.0 && inner <= 1.0);
    assert!((inner - 0.248).abs() < 0.001);
    Ok(())
}

#[test]
fn normalize_equal_weights() -> Result<(), &'static str> {
    let m1 = metric("a")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| Value01::witness((*raw)).unwrap());

    let m2 = metric("b")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| Value01::witness((*raw)).unwrap());

    let ms = score_set! { 1.0 => m1, 1.0 => m2 }?;
    let score = ms
        .score()
        .by(|(m1, m2)| m1.contribute(m1.metric().eval(0.5)) + m2.contribute(m2.metric().eval(0.5)));
    assert!((score - 0.5).abs() < 1e-10);
    Ok(())
}

#[test]
fn normalize_single_member() -> Result<(), &'static str> {
    let m = metric("only")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| Value01::witness((*raw)).unwrap());

    let ms = score_set! { 5.0 => m }?;
    let score = ms.score().by(|(m,)| m.contribute(m.metric().eval(0.7)));
    assert!((score - 0.7).abs() < 1e-10);
    Ok(())
}

#[test]
fn normalize_with_f32() -> Result<(), &'static str> {
    let m = metric("f32-m")
        .measure()
        .by(|v: f32| v)
        .map01()
        .by(|raw: &f32, _: f32| Value01::witness((*raw)).unwrap());

    let ms = score_set! { 2.0_f32 => m }?;
    let score = ms.score().by(|(m,)| m.contribute(m.metric().eval(1.0_f32)));
    assert!((score - 1.0_f32).abs() < 1e-7);
    Ok(())
}
