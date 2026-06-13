# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目

混合 Rust 库 + Python 包，名为 **score-set**。Rust crate（`src/lib.rs`）和 Python 包（`pyproject.toml`）共用相同的名称和版本号。

## 约定

- 修改 Rust 代码后，优先运行 `cargo test --locked` 验证。
- `tests_for_*` — 私有测试模块，声明在对应父抽象文件中（如 `value.rs` 内 `#[cfg(test)] #[path = "..."] mod tests_for_value;`），不在 `lib.rs` 中集中声明。
- `impls_for_*` — 当某个抽象存在可聚类的 API（如都围绕同一 trait、同一运算族），将其额外实现抽到 `impls_for_*` 模块，避免主文件膨胀。
- 测试辅助模块统一命名为 `lab`（如 `mod lab;`）。
- 优先使用工具链 CLI 生成代码（如 `cargo run -p xtask -- gen --max <N>` 生成 `gen_tuple.rs`），避免手搓可自动生成的模板。
- 修改前先给出计划，修改后给出 diff 摘要和 commit message。

## 常用命令

### Rust

```bash
cargo build                     # 构建库
cargo test --locked             # 运行所有 Rust 测试（锁定依赖）
cargo test -- <NAME>            # 按名称运行单个测试
cargo clippy                    # 代码检查
cargo fmt                       # 格式化代码
cargo run -p xtask -- gen --max 8  # 重新生成 gen_tuple.rs
```

### Python

```bash
uv sync                         # 安装依赖（添加依赖后使用）
```

## 架构

- **`src/float.rs`** — sealed `Float` trait + 公开 `ScoreFloat` trait（支持 f32/f64）。
- **`src/value.rs`** — `Value01`、`Weight`、`NormalizedWeight` witness 结构体，以及 `Contribution`、`ContributionSum`、`Score01` newtype。
- **`src/metric.rs`** — `Metric` builder（`measure().by().map01().by().build()`）。
- **`src/op.rs`** — `Op` builder（`score().by().build()`）。
- **`src/member.rs`** — `RawMember`、`Member`（含 `contribute()`）、`Members` trait（从 raw tuple → normalized tuple 的映射）。
- **`src/set.rs`** — `RawMetricSet`（不含 `T` 泛型，`T` 由 `aggregate` 方法推断）、`MetricSet`、`ScoreStage`。
- **`src/strategy.rs`** — `weighted_mean` 聚合策略。
- **`src/macros.rs`** — `score_set!` 宏（只做 raw member 包装，不选聚合策略）。
- **`src/gen_tuple.rs`** — xtask 生成的 per-arity `Members` trait 实现（arity 1 手写，2+ 用宏；feature-gated）。
- **`xtask/`** — 代码生成工具（workspace member），生成后自动 `cargo fmt`。
- **`pyproject.toml`** — Python 包声明（尚无源码）。

## 模块与测试从属关系

每个源文件声明自己的测试模块，不在 `lib.rs` 集中管理：

```
src/
  float.rs                # #[cfg(test)] #[path = "tests_for_float.rs"] mod tests_for_float;
  tests_for_float.rs
  value.rs                # #[cfg(test)] #[path = "tests_for_value.rs"] mod tests_for_value;
  tests_for_value.rs
  metric.rs               # #[cfg(test)] #[path = "tests_for_metric.rs"] mod tests_for_metric;
  tests_for_metric.rs
  op.rs                   # #[cfg(test)] #[path = "tests_for_op.rs"] mod tests_for_op;
  tests_for_op.rs
  member.rs               # #[cfg(test)] #[path = "tests_for_member.rs"] mod tests_for_member;
  tests_for_member.rs
  set.rs                  # #[cfg(test)] #[path = "tests_for_set.rs"] mod tests_for_set;
  tests_for_set.rs
  strategy.rs             # #[cfg(test)] #[path = "tests_for_strategy.rs"] mod tests_for_strategy;
  tests_for_strategy.rs
  lab.rs                  # 共享测试辅助，声明在 lib.rs
```
