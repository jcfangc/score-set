use crate::lab::gc_ratio;
use crate::*;

#[test]
fn readme_example() {
    let gc = metric("gc")
        .measure()
        .by(|dna: &str| gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &str| (*raw).witness().by(|v| Value01::prove(*v)).unwrap())
        .build();

    let len = metric("len")
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

    let ms = score_set! {
        2.0 => gc,
        3.0 => len,
    }
    .aggregate(strategy::weighted_mean)
    .unwrap();

    let dna = "ACGTACGT";

    let score = ms.score().by(|(gc, len)| {
        gc.contribute(gc.metric().eval(dna)) + len.contribute(len.metric().eval(dna.len()))
    });

    let inner = score.into_inner();
    assert!(inner >= 0.0 && inner <= 1.0);
    assert!((inner - 0.248).abs() < 0.001);
}
