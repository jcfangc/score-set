use crate::lab::gc_ratio;
use crate::*;

#[test]
fn readme_example() {
    // Build metrics
    let gc = metric("gc")
        .measure()
        .by(|dna: &str| gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &str| Value01::witness(*raw).unwrap())
        .build();

    let len = metric("len")
        .measure()
        .by(|len: usize| len)
        .map01()
        .by(|raw: &usize, _: usize| Value01::witness((*raw as f64 / 100.0).min(1.0)).unwrap())
        .build();

    // Build set with weighted_mean strategy
    let ms = score_set! {
        2.0 => gc,
        3.0 => len,
    }
    .aggregate(strategy::weighted_mean)
    .unwrap();

    let dna = "ACGTACGT";

    // Score via closure
    let score = ms.score().by(|(gc, len)| {
        gc.contribute(gc.metric().eval(dna)) + len.contribute(len.metric().eval(dna.len()))
    });

    // Verify score is in [0, 1]
    let inner = score.into_inner();
    assert!(inner >= 0.0 && inner <= 1.0);
    // With gc=0.5 for "ACGTACGT" (4/8) and normalized weights 0.4, 0.6
    // gc contribution: 0.5 * 0.4 = 0.2
    // len contribution: (8/100=0.08) * 0.6 = 0.048
    // total ≈ 0.248
    assert!((inner - 0.248).abs() < 0.001);
}
