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
- **`src/value.rs`** — `Value01`、`GtZero`、`NormalizedWeight`、`NormalizedContainer` witness。
- **`src/metric.rs`** — `Metric` builder（`metric("name").measure().by().map01().by()`）。
- **`src/member.rs`** — `RawMember`、`Member`（含 `contribute()`）、`Members` trait（从 raw tuple → normalized member tuple 的 per-arity 映射）。
- **`src/macros.rs`** — `fixed_score_set!`、`finite_score_set!`、`dynamic_score_set!` 三层宏。
- **`src/breakdown.rs`** — `Breakdown` DTO（name + score + weight + contribution）。
- **`src/fixed.rs`** — Layer 1：`FixedScoreSet<T, Members>` + `FixedScoreStage`。
- **`src/fixed_tuple.rs`** — xtask 生成，per-arity `Members` trait 实现（arity 1 手写，2+ 用 `impl_members_for_tuple!` 宏；feature-gated by `fixed-tuple-N`）。
- **`src/finite.rs`** — Layer 2：`finite_metric!` 宏（命名键语法）+ `FiniteScoreSet<T, I, E>` + `FiniteScoreSetBuilder` + `FiniteScoreStage`。
- **`src/finite_enum.rs`** — xtask 生成：匿名 `FiniteEnum1..128` + `IntoFiniteEntries` trait + `into_finite_entries()`（feature-gated by `finite-enum-N`）。
- **`src/dynamic.rs`** — Layer 3：`Scorable` trait + `DynamicScoreSet<T, I>` + `DynamicScoreSetBuilder` + `DynamicScoreStage`。
- **`xtask/`** — 代码生成（workspace member），生成 `fixed_tuple.rs`、`finite_enum.rs` 和 `Cargo.toml` `[features]`，生成后自动 `cargo fmt`。
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
  fixed.rs                # #[cfg(test)] mod tests_for_fixed;
  fixed/
    tests_for_fixed.rs
  finite.rs               # #[cfg(test)] mod tests_for_finite;
  finite/
    tests_for_finite.rs
  dynamic.rs              # #[cfg(test)] mod tests_for_dynamic;
  dynamic/
    tests_for_dynamic.rs
  lab.rs                  # 共享测试辅助，声明在 lib.rs
```
