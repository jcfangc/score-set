//! Ergonomic stress-tests for FiniteScoreSet and DynamicScoreSet.
//!
//! Simulates a real user scenario: DNA sequence scoring with GC ratio,
//! sequence length, and palindrome detection.
//!
//! Each section documents friction points from the user's perspective.
//! Broken code paths are left as doc comments to illustrate compile-failures.

use crate::*;

// ===========================================================================
// Domain: DNA sequence scoring context
// ===========================================================================

#[derive(Debug, Clone)]
struct DnaContext<'a> {
    dna: &'a str,
    len: usize,
}

impl<'a> DnaContext<'a> {
    fn new(dna: &'a str) -> Self {
        Self { dna, len: dna.len() }
    }
}

// ===========================================================================
// Part 1 — DynamicScoreSet (Layer 3) — Actually usable
// ===========================================================================

#[test]
fn dynamic_score_set_dna_example() -> Result<(), &'static str> {
    // ISSUE: Explicit Box::new() + full type annotation per metric.
    // No .into_dyn() or .boxed() convenience on the builder.
    let gc: Box<dyn DynMetric<f64, DnaContext>> = Box::new(
        metric("gc_ratio")
            .measure()
            .by(|ctx: &DnaContext| crate::lab::gc_ratio(ctx.dna))
            .map01()
            .by(|raw: &f64, _: &DnaContext| Value01::witness(raw.min(1.0).max(0.0)).unwrap()),
    );

    let len: Box<dyn DynMetric<f64, DnaContext>> = Box::new(
        metric("seq_len")
            .measure()
            .by(|ctx: &DnaContext| ctx.len as f64)
            .map01()
            .linear(100.0),
    );

    // ISSUE: new() takes complete Vec, no incremental builder, no push.
    let set = DynamicScoreSet::<f64, DnaContext>::new(vec![(2.0, gc), (1.0, len)])?;

    let ctx = DnaContext::new("ACGTACGTACGT");

    // ISSUE: .score() only does weighted sum. No access to per-metric
    // scores for debugging/explainability without manual iteration.
    let total = set.score(&ctx);
    assert!(total >= 0.0 && total <= 1.0);

    // ISSUE: Manual per-metric inspection requires .into_inner() on
    // Witnessed<T, NormalizedWeight> — an unfamiliar type to users.
    for m in set.iter() {
        let raw_weight: f64 = m.weight.into_inner();
        let score = m.metric().eval(&ctx);
        let _contribution = raw_weight * (*score);
    }

    Ok(())
}

// ===========================================================================
// Part 2 — Side-by-side verbosity comparison
// ===========================================================================

#[test]
fn compare_layer_verbosity() -> Result<(), &'static str> {
    // Layer 1: 1 line macro call.
    let _total = score_set! {
        2.0 => metric("a").measure().by::<f64, f64, _>(|v: &f64| *v).map01().identity(),
        3.0 => metric("b").measure().by::<f64, f64, _>(|v: &f64| *v).map01().identity(),
    }?
    .score()
    .by(|(a, b)| a.contribute(a.metric().eval(&0.5)) + b.contribute(b.metric().eval(&0.5)));

    // Layer 3: 2 type annotations, 2 explicit Box::new, 1 explicit Vec.
    let a: Box<dyn DynMetric<f64, f64>> = Box::new(
        metric("a").measure().by(|v: &f64| *v).map01().identity(),
    );
    let b: Box<dyn DynMetric<f64, f64>> = Box::new(
        metric("b").measure().by(|v: &f64| *v).map01().identity(),
    );
    let _dynamic = DynamicScoreSet::<f64, f64>::new(vec![(2.0, a), (3.0, b)])?;

    // ISSUE: Layer 3 needs ~3x the lines for the same operation.
    // Layer 2 would need even more (newtype definitions per metric).

    Ok(())
}
