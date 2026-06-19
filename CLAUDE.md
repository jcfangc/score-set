# score-set

混合 Rust 库 + Python 包。Rust crate（`src/lib.rs`）和 Python 包（`pyproject.toml`）共用相同的名称和版本号。

通用约定参见父级：`../CLAUDE.md`

## 项目特有命令

```bash
cargo run -p xtask -- gen --max 8  # 重新生成 fixed_tuple.rs, finite_enum.rs 和 Cargo.toml features
uv sync                            # 安装 Python 依赖
```

## 架构

- **`src/float.rs`** — sealed `Float` trait + 公开 `ScoreFloat` trait（支持 f32/f64）。
- **`src/value.rs`** — `Value01`、`Weight`、`NormalizedWeight` witness 结构体，以及 `Contribution`、`ContributionSum`、`Score01` newtype。
- **`src/metric.rs`** — `Metric` builder（`measure().by().map01().by()`）。
- **`src/member.rs`** — `RawMember`、`Member`（含 `contribute()`）、`Members` trait（从 raw tuple → normalized tuple 的映射）。
- **`src/set.rs`** — `RawMetricSet`（不含 `T` 泛型，`T` 由 `aggregate` 方法推断）、`MetricSet`、`ScoreStage`。
- **`src/strategy.rs`** — `weighted_mean` 聚合策略。
- **`src/macros.rs`** — `score_set!` 宏（只做 raw member 包装，不选聚合策略）。
- **`src/fixed_tuple.rs`** — xtask 生成的 per-arity `Members` trait 实现（arity 1 手写，2+ 用宏；feature-gated by `fixed-tuple-N`）。
- **`src/finite_enum.rs`** — xtask 生成的匿名零虚表枚举 `FiniteEnum1`..`FiniteEnumN`，供 `finite_score_set!` 宏使用（feature-gated by `finite-enum-N`）。
- **`xtask/`** — 代码生成工具（workspace member），生成后自动 `cargo fmt`。
- **`pyproject.toml`** — Python 包声明（尚无源码）。

## 模块与测试从属关系

```
src/
  float.rs                # #[cfg(test)] mod tests_for_float;
  float/
    tests_for_float.rs
  value.rs                # #[cfg(test)] mod tests_for_value;
  value/
    tests_for_value.rs
  metric.rs               # #[cfg(test)] mod tests_for_metric;
  metric/
    tests_for_metric.rs
  member.rs               # #[cfg(test)] mod tests_for_member;
  member/
    tests_for_member.rs
  set.rs                  # #[cfg(test)] mod tests_for_set;
  set/
    tests_for_set.rs
  strategy.rs             # #[cfg(test)] mod tests_for_strategy;
  strategy/
    tests_for_strategy.rs
  lab.rs                  # 共享测试辅助，声明在 lib.rs
```
