//! Tests for precision coexistence.
//!
//! These tests verify that in `both` mode, `ScoreSet32`/`ScoreSet64` and their
//! associated types coexist without name conflicts, produce values of the
//! correct types, and remain type-safe (no accidental mixing).
//!
//! This module compiles only when `--features both` is passed.

use score_set::*;

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

struct Ctx {
    a: f32,
    b: f64,
}

// ---------------------------------------------------------------------------
// Type identity
// ---------------------------------------------------------------------------

#[test]
fn score32_is_f32() {
    assert_eq!(core::mem::size_of::<Score32>(), 4);
}

#[test]
fn score64_is_f64() {
    assert_eq!(core::mem::size_of::<Score64>(), 8);
}

// ---------------------------------------------------------------------------
// Coexistence — both families usable in the same scope
// ---------------------------------------------------------------------------

#[test]
fn both_families_evaluate() -> Result<(), &'static str> {
    // f32 family
    let m32 = metric32("a")
        .measure()
        .by(|ctx: &Ctx| ctx.a)
        .map01()
        .identity();

    let s32: f32 = ScoreSet32::new().push(1.0, m32)?.sum()?(&Ctx { a: 0.5, b: 0.0 });

    // f64 family
    let m64 = metric64("b")
        .measure()
        .by(|ctx: &Ctx| ctx.b)
        .map01()
        .identity();

    let s64: f64 = ScoreSet64::new().push(1.0, m64)?.sum()?(&Ctx { a: 0.0, b: 0.75 });

    assert!((s32 - 0.5).abs() < 1e-6);
    assert!((s64 - 0.75).abs() < 1e-9);
    Ok(())
}

#[test]
fn both_families_breakdown() -> Result<(), &'static str> {
    let m32 = metric32("x")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    let rows: Vec<Breakdown32> = ScoreSet32::new()
        .push(2.0, m32)?
        .breakdown(&0.5)?
        .into_iter()
        .collect();
    // Verify field types are f32 (compiles == correct)
    let _: f32 = rows[0].score;
    let _: f32 = rows[0].weight;
    let _: f32 = rows[0].contribution;

    let m64 = metric64("y")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let rows: Vec<Breakdown64> = ScoreSet64::new()
        .push(3.0, m64)?
        .breakdown(&0.6)?
        .into_iter()
        .collect();
    // Verify field types are f64
    let _: f64 = rows[0].score;
    let _: f64 = rows[0].weight;
    let _: f64 = rows[0].contribution;

    Ok(())
}

#[test]
fn both_families_multiple_metrics() -> Result<(), &'static str> {
    let m1 = metric32("a")
        .measure()
        .by(|ctx: &Ctx| ctx.a)
        .map01()
        .linear(100.0);

    let m2 = metric32("a2")
        .measure()
        .by(|ctx: &Ctx| 1.0 - ctx.a)
        .map01()
        .identity();

    let total: f32 =
        ScoreSet32::new().push(1.0, m1)?.push(2.0, m2)?.sum()?(&Ctx { a: 0.3, b: 0.0 });

    let m3 = metric64("b")
        .measure()
        .by(|ctx: &Ctx| ctx.b)
        .map01()
        .inc_sigmoid(0.0, 1.0);

    let total64: f64 = ScoreSet64::new().push(5.0, m3)?.sum()?(&Ctx { a: 0.0, b: 0.7 });

    assert!(total > 0.0);
    assert!(total64 > 0.0);
    Ok(())
}
