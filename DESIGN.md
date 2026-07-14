# score-set DSL 编译器重构 — 探索笔记

## 最终结论

**库的核心定位：有限评分空间生成器（finite scoring space generator）**

不是运行时评分框架，而是编译期笛卡尔积展开器。接受有限 Measure × Map 声明，生成
Rust 类型、proto schema、builder、无分支 ScoreSet。

## 架构

```
score_space! { measures {...} maps {...} }
        │
        ▼  (编译期生成)
┌──────────────────────────────────────────┐
│  每个 Measure×Map 组合:                   │
│  - Rust 类型: Metric<M, P, Ctx>          │
│  - Proto message: GcLinearMsg {...}      │
│  - ScoreConfig: 聚合所有 optional 字段    │
│  - ScoreSet: 所有组合 + 权重, 无分支score │
│  - ScoreSet::build(config): 构造阶段      │
│  - ScoreSet::score(&ctx): 热路径纯算术    │
└──────────────────────────────────────────┘
```

## 经历过的错误方向

### 错误 1: 闭包擦除类型身份

`Metric32<C, F: Fn(&C) -> f32>` 把 measure 表达为闭包。问题：闭包没有类型名，
无法构造 StaticMetric enum，无法做 match dispatch。

**教训**：需要 `M: Measure<C>` trait + 具体 struct。

### 错误 2: StaticMetric enum 在热路径

`Vec<StaticMetric>` + `match self { 9 arms }` 每次 score 调用都做分支预测。
对于百万次调用 × 多个 metric，branch misprediction 不可忽略。

**教训**：enum match dispatch 应该只在构造阶段（build），不应在热路径（score）。

### 错误 3: 追求"proto 动态配置 → 运行时生成新类型"

想用 `Box<dyn Builder>` → 运行时生成 `struct GcMfeScorer { ... }`。
但 Rust 不能在运行时创造新类型——类型必须在编译期存在。

**教训**：动态配置不能产生编译期类型。只能提前生成所有可能类型，运行时选择字段。

### 错误 4: 参数 vs 类型混淆

`Linear { min: f32, max: f32 }` 有 2^64 种参数组合。不可能为每种参数生成一个类型。
但参数是**值空间**，不是**类型空间**。`Metric<Gc, Linear>` 是类型，`min=0, max=1`
是值。

**教训**：Measure×Map = 类型空间（编译期展开）。Params = 值空间（运行时数据）。
Metric 同时持有 measure 和 map 的**值**。

### 错误 5: toy domain 太简单

第一版测试用 X/Y/Z (ZST) × Identity/Positive/Inverse (ZST)，所有 Metric 都是
零大小类型。虽能证明 dispatch 模式，但不能证明带参数 Measure (Mfe { temperature })
和带参数 Map (Cauchy { center }) 的正确性。

**教训**：测试必须包含参数化类型，否则无法验证参数传递和存储。

## 当前正确方案

### 三个阶段

| 阶段 | 频率 | 允许 | 禁止 |
|------|------|------|------|
| Proto decode | 每次配置变更 | match, enum, Option, Vec, HashMap | — |
| ScoreSet::build | 每次配置变更 | match, if let, 参数验证, 权重归一化 | — |
| ScoreSet::score | 百万次 | 纯算术: `a*w + b*w + ...` | match, enum, if, dyn, Box, fn ptr |

### ScoreSet 结构

```rust
pub struct ScoreSet {
    // 所有 Measure×Map 组合，每个是一个 named field
    gc_linear: Metric<Gc, Linear, Ctx>,
    gc_cauchy: Metric<Gc, Cauchy, Ctx>,
    tm_linear: Metric<Tm, Linear, Ctx>,
    // ...

    // 权重：活跃 slot 有正权重，不活跃 = 0.0
    w_gc_linear: f32,
    w_gc_cauchy: f32,
    w_tm_linear: f32,
    // ...
}

impl ScoreSet {
    #[inline]
    pub fn score(&self, ctx: &Ctx) -> f32 {
        self.gc_linear.score(ctx) * self.w_gc_linear
        + self.gc_cauchy.score(ctx) * self.w_gc_cauchy
        + self.tm_linear.score(ctx) * self.w_tm_linear
        // ... 纯算术, 零分支
    }
}
```

### 代价与取舍

- **Proto schema 膨胀**: 100 个 Measure × 5 个 Map = 500 个 proto message。
  但这不是 bug——proto 本身成为能力声明和文档。
- **ScoreSet 体积**: 500 个字段可能几十 KB。但对每个 scorer 实例是一次性分配，
  且热路径受益远大于内存开销。
- **不开放 Custom Map**: Map 是数学空间（Linear, Cauchy, Sigmoid），不是业务空间。
  库维护者控制 Map 闭集合，开发者只选择。

### 为什么不用 dyn/Box

`dyn Measure` 擦除了类型→无法内联→每次调用 indirect call。
Measure 如果很重（MFE O(n²)），一次 indirect call 可以接受。
但如果 Measure 很轻（field access），vtable 开销不可忽视。
本方案在两极之间：重 Measure 的参数已经存在结构体里，轻 Measure 完全内联。

## 测试文件结构

```
tests/cartesian_property.rs              # 测试根: Sample, proptest
tests/cartesian_property/
  lib_internals.rs      # Measure trait, Map01 trait, builtin maps, Metric<M,P,Ctx>
  dev_declaration.rs    # 模拟宏输出: X/Y/OffsetZ, 9 config msg, ScoreConfig, ScoreSet, reference oracle
  proto_caller.rs       # 最简调用方: UserConfig, score_batch, 不 import 内部类型
```

测试覆盖:
- `smoke_test_all_9_variants` — 手工枚举 9 个 variant
- `score_set_equals_reference` — proptest: 随机 config → builder → branchless score == oracle
- `proto_caller_works_without_internals` — 编译期验证 proto_caller 不依赖内部类型
- `score_set_is_compact` — ScoreSet 小于 256 bytes

## 关键洞察

1. **有限空间显式展开比动态配置更工程化**：承认「这就是全部可能」，比假装可以动态扩展更诚实。
2. **dispatch 只在构造阶段**：match/if let 只存在于 build() 中（低频），score() 是纯算术（高频）。
3. **类型空间 ≠ 值空间**：Measure×Map = 编译期类型。参数 = 运行时数据。不混淆。
4. **proto message per combination**：每个组合一个 message，proto 即文档。
5. **OUT_DIR 模式**：build.rs / proc-macro 生成代码，类似 prost/tonic 模式。
