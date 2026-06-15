# Ergonomics Report — FiniteScoreSet & DynamicScoreSet

## 测试方法

用一个 DNA 序列评分场景（GC 含量、序列长度、回文检测三个算子）分别
走 Layer 2 和 Layer 3 的完整流程，从用户视角记录每一步的摩擦点。

完整用例代码见 `src/lab/ergonomics.rs`，可通过 `cargo test` 编译观察实际
错误信息。

---

## 一、FiniteScoreSet（Layer 2）

### 问题 1：名类型要求迫使用户绕路

`finite_metric!` 要求每个 variant 的类型在 Rust 中**可被命名**
（nameable），以便生成 `enum Foo { Variant(TheType) }`。

但用户最自然的做法——用 `metric().measure().by(closure).map01().by(closure)`
构造的 `Metric<T,I,Raw,M,F>` —— **不可命名**，因为 `M` 和 `F` 是匿名闭包类型。

三条迂回路线：

| 路线 | 做法 | 代价 |
|---|---|---|
| fn 指针 | 把闭包写成顶层 fn | 不能捕获变量；且 Metric 推断出的仍是唯一闭包类型，需显式标注 `as fn(...)` |
| newtype 封装 | `struct Gc(Metric<...>)` + 手写 `eval`/`name` 代理 | 每个指标 ~15 行样板代码 |
| 直接实现 | 不用 Metric，手写 `eval`/`name` | 放弃 Metric builder 生态 |

**实测**：即使用 fn 指针，Metric builder 仍会将 `measure_gc` 推断为
`fn(&DnaContext<'b>) -> f64 {measure_gc}` 这种唯一闭包类型，而非
`fn(&DnaContext) -> f64`。用户必须显式标注 `as fn(...)` 强制转换。

### 问题 2：泛型 `<T, I>` 与实际用例脱节

`finite_metric!` 生成泛型枚举：
```rust
enum DnaMetricKind<T: Float, I> { ... }
```

但用户定义的指标类型（如 `GcRatio`）几乎总是**具体类型**——
`T = f64`、`I = DnaContext<'static>` 已经固定。泛型参数在这里是**多余的**，
反而导致类型不匹配：

```
error[E0308]: mismatched types
  expected `&DnaContext<'static>`, found `&I`
```

用户需要 `finite_metric! { Name => ... }` 这种非泛型语法，但目前不支持。

### 问题 3：newtype 样板代码量大

每个指标至少需要：
- struct 定义
- 构造函数
- `eval` 代理方法
- `name` 代理方法

两个指标 ≈ 30 行纯样板。五个指标 ≈ 75 行。这是采用 Layer 2 的最大门槛。

### 问题 4：宏内错误信息不可读

如果 variant 类型缺少 `eval` 或 `name`，错误指向宏展开代码而非用户类型：
```
error[E0308]: mismatched types
   --> src/finite.rs:76:51    <-- 宏内部，不是用户代码
```

---

## 二、DynamicScoreSet（Layer 3）

### 问题 5：显式装箱 + 类型标注冗余

每个指标需要：
```rust
let gc: Box<dyn DynMetric<f64, DnaContext>> = Box::new(
    metric("gc_ratio")
        .measure().by(|ctx: &DnaContext| ...)
        .map01().by(|raw: &f64, _: &DnaContext| ...)
);
```

`Box::new(...)` + 类型标注 `Box<dyn DynMetric<f64, DnaContext>>` 每个指标
都要写一遍。缺一个 `.into_dyn()` 或 `.boxed()` 便捷方法。

### 问题 6：`new(Vec<(T, Box<...>)>)` 不支持增量构建

`DynamicScoreSet::new` 接受完整 `Vec`。用户不能：
- 先创建空集，再 `.push(weight, metric)`
- 从已有集合派生（添加/移除指标）
- 用 builder 模式链式构造

对比 Layer 1 的 `score_set! { w => m, ... }` 宏语法，差距明显。

### 问题 7：`.score()` 只做加权求和

没有：
- 获取各指标单独得分后再聚合的能力
- 自定义聚合函数（min、max、几何平均）
- 条件跳过某个指标

用户要检查各指标贡献必须手动 iterate —— 可行但不便利。

### 问题 8：权重值被 `Witnessed` 封装

`member.weight` 类型是 `Witnessed<T, NormalizedWeight>`，用户需调用
`.into_inner()` 才能拿到裸 `T` 值。`Witnessed` 库对外部用户是陌生概念，
该类型未在库文档中充分解释。

---

## 三、跨层问题

### 问题 9：三层构造方式不一致

| Layer | 构造方式 | 代码行数 |
|---|---|---|
| 1 | `score_set! { w => m, ... }` | 1 |
| 2 | `FiniteScoreSet::<T, I, E>::new(vec![...])` | 3-5 + newtype 定义 |
| 3 | `DynamicScoreSet::<T, I>::new(vec![...])` | 3-5 + 装箱标注 |

Layer 1 有宏语法糖；Layer 2/3 只有裸构造函数。一致性差。

### 问题 10：`I` 类型强制统一，库不提供 context 聚合模式

所有指标必须接受同一个 `&I` 参数。当不同指标需要不同类型输入时（如 GC
需要 `&str`，长度需要 `usize`），用户必须自己发明 `DnaContext` 这样的
聚合结构体。库可以提供模式建议，但不提供任何辅助。

### 问题 11：`.metric()` 返回类型不一致

| 类型 | `.metric()` 返回 |
|---|---|
| `Member<T, M>` | `&M` |
| `FiniteMember<T, E>` | `&E` |
| `DynamicMember<T, I>` | `&dyn DynMetric<T, I>` |

三者都是 "访问 metric"，但返回的具体形状不同。这会增加用户在不同层之间
迁移时的心智负担。

### 问题 12：层间无迁移路径

从 Layer 1 迁移到 Layer 2/3 需要**重写全部指标定义**——结构体、代理方法、
枚举声明全部推倒重来。没有 `ScoreSet::into_finite()` 或
`Metric::into_dyn()` 这样的桥接。

---

## 四、做得好的地方

- **命名体系清晰**：`fixed` → `finite` → `dynamic`，单概念词无冗余
- **`DynMetric` 好读**：`Box<dyn DynMetric<f64, &str>>` 自然通顺
- **`finite_metric!` 语法干净**：`Name<T,I> => Variant(Type), ...`
- **`Custom` escape hatch**：枚举中保留 `Box<dyn DynMetric>` 兜底口，设计合理
- **三层共享 contribute 模式**：`member.contribute(score)` 跨层一致
- **归一化逻辑统一**：三层用同一套 NormalizedContainer/NormalizedWeight

---

## 五、改进建议（按优先级）

| 优先级 | 建议 | 影响范围 |
|---|---|---|
| P0 | `finite_metric!` 支持非泛型语法 `Name => Var(Ty), ...` | Layer 2 |
| P0 | Metric builder 加 `.boxed()` → `Box<dyn DynMetric<T,I>>` | Layer 3 |
| P1 | DynamicScoreSet 加 `.push(weight, metric)` 增量构建 | Layer 3 |
| P1 | FiniteScoreSet/DynamicScoreSet 加 `scores(&I) -> Vec<(name, weight, score)>` 调试方法 | Layer 2/3 |
| P2 | 提供 `finite_metric_set!` 和 `dynamic_score_set!` 宏语法糖 | Layer 2/3 |
| P2 | 提供 `impl DynMetric for ...` 的 derive 宏（减少 newtype 样板） | Layer 2 |
| P3 | 文档中添加 `DnaContext` 聚合模式示例 | 文档 |
| P3 | `.score_with(Fn(acc, contribution) -> T)` 自定义聚合 | Layer 2/3 |
