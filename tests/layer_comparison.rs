// ===========================================================================
// 三层完整流程对比：构建 Metric → 构建 MetricSet → 评分
// 场景：对 DNA 序列打分，两个指标 — GC 含量 + 序列长度
// ===========================================================================

use score_set::*;

// ---- 测度函数（fn 指针，可命名，能放入枚举变体） ----

fn measure_gc(dna: &String) -> f64 {
    if dna.is_empty() {
        return 0.0;
    }
    let gc = dna.chars().filter(|c| *c == 'G' || *c == 'C').count();
    gc as f64 / dna.len() as f64
}

fn map01_clamp(raw: &f64, _: &String) -> Witnessed<f64, Value01> {
    Value01::witness(raw.min(1.0).max(0.0)).unwrap()
}

fn measure_len(dna: &String) -> f64 {
    dna.len() as f64
}

fn map01_linear_100(raw: &f64, _: &String) -> Witnessed<f64, Value01> {
    Value01::witness((raw / 100.0).min(1.0).max(0.0)).unwrap()
}

type GcMetric =
    Metric<f64, String, f64, fn(&String) -> f64, fn(&f64, &String) -> Witnessed<f64, Value01>>;
type LenMetric =
    Metric<f64, String, f64, fn(&String) -> f64, fn(&f64, &String) -> Witnessed<f64, Value01>>;

fn gc_metric() -> GcMetric {
    metric("gc")
        .measure()
        .by(measure_gc as fn(&String) -> f64)
        .map01()
        .by(map01_clamp)
}

fn len_metric() -> LenMetric {
    metric("len")
        .measure()
        .by(measure_len as fn(&String) -> f64)
        .map01()
        .by(map01_linear_100)
}

// ===========================================================================
// Layer 1 — 编译期固定：fixed_score_set! 宏
// ===========================================================================

#[test]
fn layer1_full_flow() -> Result<(), &'static str> {
    let set = fixed_score_set! {
        2.0 => gc_metric(),
        3.0 => len_metric(),
    }?;

    let dna = "ACGTACGTACGT".to_string();
    let total = set.score().by(|(gc, len)| {
        gc.contribute(gc.metric().eval(&dna)) + len.contribute(len.metric().eval(&dna))
    });

    let expected = 0.4 * 0.5 + 0.6 * 0.12;
    assert!((total - expected).abs() < 1e-10);

    let worst = set.score().by(|(gc, len)| {
        let a = gc.contribute(gc.metric().eval(&dna));
        let b = len.contribute(len.metric().eval(&dna));
        a.min(b)
    });
    assert!((worst - 0.6 * 0.12).abs() < 1e-10);

    Ok(())
}

// ===========================================================================
// Layer 2 — 有限枚举：finite_score_set! 宏（无需声明枚举！）
// ===========================================================================

#[test]
fn layer2_full_flow() -> Result<(), &'static str> {
    let set = finite_score_set! {
        2.0 => gc_metric(),
        3.0 => len_metric(),
    }?;

    let dna = "ACGTACGTACGT".to_string();
    let total = set.score().by(|members| {
        members
            .iter()
            .fold(0.0, |acc, m| acc + m.contribute(m.metric().eval(&dna)))
    });

    let expected = 0.4 * 0.5 + 0.6 * 0.12;
    assert!((total - expected).abs() < 1e-10);

    let via_sum = set.sum(&dna);
    assert!((total - via_sum).abs() < 1e-10);

    let worst = set.score().by(|members| {
        members
            .iter()
            .map(|m| m.contribute(m.metric().eval(&dna)))
            .fold(f64::INFINITY, f64::min)
    });
    assert!((worst - 0.6 * 0.12).abs() < 1e-10);

    let gc_only = set.score().by(|members| {
        members
            .iter()
            .filter(|m| m.metric().name().contains("gc"))
            .fold(0.0, |acc, m| acc + m.contribute(m.metric().eval(&dna)))
    });
    assert!((gc_only - 0.4 * 0.5).abs() < 1e-10);

    Ok(())
}

// ===========================================================================
// Layer 3 — 完全动态：dynamic_score_set! 宏（无需手动 .boxed()！）
// ===========================================================================

#[test]
fn layer3_full_flow() -> Result<(), &'static str> {
    let set = dynamic_score_set! {
        2.0 => gc_metric(),
        3.0 => len_metric(),
    }?;

    let dna = "ACGTACGTACGT".to_string();
    let total = set.score().by(|members| {
        members
            .iter()
            .fold(0.0, |acc, m| acc + m.contribute(m.metric().eval(&dna)))
    });

    let expected = 0.4 * 0.5 + 0.6 * 0.12;
    assert!((total - expected).abs() < 1e-10);

    let via_sum = set.sum(&dna);
    assert!((total - via_sum).abs() < 1e-10);

    let geo = set.score().by(|members| {
        members
            .iter()
            .map(|m| m.contribute(m.metric().eval(&dna)))
            .fold(1.0, |a, c| a * c)
    });
    let expected_geo = (0.4 * 0.5) * (0.6 * 0.12);
    assert!((geo - expected_geo).abs() < 1e-10);

    Ok(())
}
