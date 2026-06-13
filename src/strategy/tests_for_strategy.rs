use crate::*;

#[test]
fn weighted_mean_normalizes() {
    let a = metric("a")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    let b = metric("b")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    let ms = score_set! {
        2.0 => a,
        3.0 => b,
    }
    .aggregate(strategy::weighted_mean)
    .unwrap();

    let score = ms
        .score()
        .by(|(a, b)| a.contribute(a.metric().eval(1.0)) + b.contribute(b.metric().eval(1.0)));

    assert!((score.into_inner() - 1.0).abs() < 1e-10);
}

#[test]
fn weighted_mean_rejects_zero_sum() {
    let m = metric("m")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    let m2 = metric("m2")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    assert!(
        score_set! {
            0.0_f64 => m,
            0.0_f64 => m2,
        }
        .aggregate(strategy::weighted_mean::<f64, (Member<f64, _>, Member<f64, _>)>)
        .is_err()
    );
}

#[test]
fn weighted_mean_equal_weights() {
    let m1 = metric("m1")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    let m2 = metric("m2")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    let ms = score_set! {
        1.0 => m1,
        1.0 => m2,
    }
    .aggregate(strategy::weighted_mean)
    .unwrap();

    let score = ms
        .score()
        .by(|(m1, m2)| m1.contribute(m1.metric().eval(0.5)) + m2.contribute(m2.metric().eval(0.5)));

    assert!((score.into_inner() - 0.5).abs() < 1e-10);
}

#[test]
fn weighted_mean_single_member() {
    let m = metric("only")
        .measure()
        .by(|v: f64| v)
        .map01()
        .by(|raw: &f64, _: f64| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    let ms = score_set! {
        5.0 => m,
    }
    .aggregate(strategy::weighted_mean)
    .unwrap();

    let score = ms.score().by(|(m,)| m.contribute(m.metric().eval(0.7)));

    assert!((score.into_inner() - 0.7).abs() < 1e-10);
}

#[test]
fn weighted_mean_with_f32() {
    let m = metric("f32-m")
        .measure()
        .by(|v: f32| v)
        .map01()
        .by(|raw: &f32, _: f32| (*raw).witness().by(Value01::prove()).unwrap())
        .build();

    let ms = score_set! {
        2.0_f32 => m,
    }
    .aggregate(strategy::weighted_mean)
    .unwrap();

    let score = ms.score().by(|(m,)| m.contribute(m.metric().eval(1.0_f32)));

    assert!((score.into_inner() - 1.0_f32).abs() < 1e-7);
}
