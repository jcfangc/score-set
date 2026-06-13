use crate::*;

#[test]
fn weighted_mean_normalizes() -> Result<(), &'static str> {
    let a = metric("a")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let b = metric("b")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let ms = score_set! {
        2.0 => a,
        3.0 => b,
    }?
    .aggregate(strategy::weighted_mean)?;

    let score = ms
        .score()
        .by(|(a, b)| a.contribute(a.metric().eval(1.0)) + b.contribute(b.metric().eval(1.0)));

    assert!((score - 1.0).abs() < 1e-10);
    Ok(())
}

#[test]
fn weighted_mean_rejects_zero_sum() {
    assert!(raw_member(0.0_f64, "x").is_err());
}

#[test]
fn weighted_mean_equal_weights() -> Result<(), &'static str> {
    let m1 = metric("m1")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let m2 = metric("m2")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let ms = score_set! {
        1.0 => m1,
        1.0 => m2,
    }?
    .aggregate(strategy::weighted_mean)?;

    let score = ms
        .score()
        .by(|(m1, m2)| m1.contribute(m1.metric().eval(0.5)) + m2.contribute(m2.metric().eval(0.5)));

    assert!((score - 0.5).abs() < 1e-10);
    Ok(())
}

#[test]
fn weighted_mean_single_member() -> Result<(), &'static str> {
    let m = metric("only")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let ms = score_set! {
        5.0 => m,
    }?
    .aggregate(strategy::weighted_mean)?;

    let score = ms.score().by(|(m,)| m.contribute(m.metric().eval(0.7)));

    assert!((score - 0.7).abs() < 1e-10);
    Ok(())
}

#[test]
fn weighted_mean_with_f32() -> Result<(), &'static str> {
    let m = metric("f32-m")
        .measure()
        .by(|v: f32| v)
        .map01()
        .by(|raw: &f32, _: f32| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let ms = score_set! {
        2.0_f32 => m,
    }?
    .aggregate(strategy::weighted_mean)?;

    let score = ms.score().by(|(m,)| m.contribute(m.metric().eval(1.0_f32)));

    assert!((score - 1.0_f32).abs() < 1e-7);
    Ok(())
}
