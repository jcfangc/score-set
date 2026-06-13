# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目

混合 Rust 库 + Python 包，名为 **score-set**（早期阶段）。Rust crate（`src/lib.rs`）和 Python 包（`pyproject.toml`）共用相同的名称和版本号。

## 约定

- 修改 Rust 代码后，优先运行 `cargo test --locked` 验证。
- 测试模块统一命名为 `tests_for_*`（如 `tests_for_add`）。
- 测试辅助模块统一命名为 `lab`（如 `mod lab;`）。
- 修改前先给出计划，修改后给出 diff 摘要和 commit message。

## 常用命令

### Rust

```bash
cargo build           # 构建库
cargo test --locked   # 运行所有 Rust 测试（锁定依赖）
cargo test -- <NAME>  # 按名称运行单个测试
cargo clippy          # 代码检查
cargo fmt             # 格式化代码
```

### Python

```bash
uv sync               # 安装依赖（添加依赖后使用）
```

## 架构

- **`src/lib.rs`** — Rust 库。目前仅包含一个 `add` 函数和一个单元测试。使用 Rust 2024 版。
- **`pyproject.toml`** — Python 包声明（尚无源码）。要求 Python ≥ 3.11。
- Rust 和 Python 包可独立构建；它们共用项目根目录仅为方便，目前两者之间尚无绑定层。
