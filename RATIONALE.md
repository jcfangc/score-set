# Design Rationale

A score is modeled as a weighted sum of metrics. For a context $x$, each metric consists of a measurement $m$, a mapping $g$, and a weight $w$:

$$
\operatorname{metric}(x)
=
w,g(m(x))
$$

A score set containing $n$ metrics evaluates:

$$
\operatorname{score}(x)
=
\sum_{i=1}^{n}
w_i g_i(m_i(x))
$$

At the mathematical level, every metric has the same structure:

$$
X
\xrightarrow{m_i}
\mathbb{R}
\xrightarrow{g_i}
[0,1]
\xrightarrow{\times w_i}
\mathbb{R}
$$

The implementation challenge is therefore not the scoring equation itself. It is how to represent a runtime-selected collection of heterogeneous implementations in a statically typed language.

## The original objective

The initial design attempted to satisfy all of the following properties simultaneously:

1. The set of metrics may be selected at runtime.
2. Only selected metrics are stored and evaluated.
3. Each measurement and mapping remains a concrete generic type.
4. The complete evaluation path is statically dispatched.
5. Evaluation performs no runtime type branch or indirect call.
6. The resulting score set has one stable type that can be returned and stored.
7. The implementation does not enumerate every possible metric subset.
8. The implementation does not generate machine code at runtime.

Each property is individually reasonable. The difficulty is that they are not jointly satisfiable in ordinary ahead-of-time Rust.

## Runtime selection changes type structure

Assume that $Z$ is an empty evaluator and that $A$ and $B$ are concrete metrics.

A statically composed evaluator may have one of the following structures:

$$
E_{\varnothing}=Z
$$

$$
E_{{A}}=\operatorname{Append}(Z,A)
$$

$$
E_{{B}}=\operatorname{Append}(Z,B)
$$

$$
E_{{A,B}}
=
\operatorname{Append}
\bigl(
\operatorname{Append}(Z,A),
B
\bigr)
$$

Although all four values implement the same evaluation operation, they are four different concrete types.

For $N$ independently optional metrics, the runtime configuration may select any member of the power set:

$$
\mathcal{P}(\mathcal{M})
$$

where $\mathcal{M}$ is the set of available metrics. The number of possible subsets is:

$$
\left|\mathcal{P}(\mathcal{M})\right|
=
2^N
$$

If the type of an evaluator directly encodes its selected metrics, runtime selection therefore implies up to $2^N$ different concrete evaluator types.

## A stable return type requires unification

A normal function has one return type determined at compile time:

$$
\operatorname{compile}
:
\operatorname{Configuration}
\rightarrow
T
$$

However, a runtime configuration may produce:

$$
T_0,\ T_1,\ \ldots,\ T_k
$$

where the selected $T_i$ is not known until execution.

Return-position `impl Trait` does not change this requirement. It hides the name of one concrete type, but the hidden type must still be identical for every return path.

The central conflict can be expressed as:

$$
\boxed{
\text{runtime-selected type structure}
+
\text{one concrete result type}
+
\text{no type unification mechanism}
}
$$

These conditions cannot all hold simultaneously.

A runtime-selected heterogeneous value must eventually be represented using some common structure.

## Where the unavoidable cost can be placed

There are several general ways to unify the possible evaluator structures. Each places the necessary cost in a different part of the system.

### Type erasure

Different concrete evaluators can be represented through a common interface.

Conceptually:

$$
\exists E.;
E \land \operatorname{Eval}(E)
$$

The concrete type $E$ still exists, but callers interact with it through an erased representation.

This preserves arbitrary runtime composition and avoids enumerating all subsets. The cost is an indirect operation when the erased evaluator is invoked.

### Sum types

All supported concrete alternatives can be represented as variants of one finite sum type:

$$
T
=

T_1+T_2+\cdots+T_k
$$

Evaluation inspects the active variant and then invokes the corresponding concrete implementation.

This preserves concrete implementations inside each branch. The cost is a runtime tag selection, and the representation must be updated when the closed set of alternatives changes.

### Fixed product types

All possible metric slots can be represented simultaneously:

$$
T
=

T_1\times T_2\times\cdots\times T_N
$$

Runtime configuration changes the values or activation state of these slots rather than changing the type.

This gives one concrete static structure, but sparse configurations retain the complete product. The cost appears as inactive storage, activation checks, or unnecessary evaluation.

### Enumeration of all subsets

Every possible subset can be assigned its own concrete evaluator and then included in one top-level representation.

The number of cases is:

$$
2^N
$$

This can move runtime selection outside the inner evaluation path, but compile time, code size, and maintenance cost grow exponentially.

### Runtime code generation

A runtime configuration can be translated into a specialized native function.

This removes per-metric representation dispatch from the resulting evaluation path, but changes the problem from type composition to runtime compilation.

The cost is no longer a branch or indirect call inside the score loop. It becomes runtime compiler complexity, executable-memory management, platform support, and compilation latency.

## The impossible combination

The original design was effectively searching for a representation with all of these properties:

$$
\begin{aligned}
&\text{arbitrary runtime subset selection},\
&\text{only selected metrics evaluated},\
&\text{one storable result type},\
&\text{fully static dispatch},\
&\text{no exhaustive subset generation},\
&\text{no runtime code generation}.
\end{aligned}
$$

For arbitrary heterogeneous metric types, no ordinary AOT representation provides all six.

A difference selected at runtime must remain observable somewhere. It may appear as:

* a type tag;
* an indirect function call;
* an activation state;
* an enumerated combination;
* or generated machine code.

The distinction cannot be removed. It can only be relocated.

## Static and dynamic composition are different problems

When a metric composition is known at compile time, its structure can be encoded directly in its concrete type:

$$
\operatorname{StaticScore}
=
T_1\times T_2\times\cdots\times T_n
$$

The compiler knows the complete call graph and may monomorphize and inline it.

When the composition is selected at runtime, the program instead needs a value-level representation:

$$
\operatorname{DynamicScore}
=
[T_{i_1},T_{i_2},\ldots,T_{i_k}]
$$

where the indices $i_1,\ldots,i_k$ are determined by runtime data.

The two cases share the same mathematical scoring model, but they do not have the same type-level information.

This distinction is fundamental:

$$
\text{compile-time-known composition}
\neq
\text{runtime-selected composition}
$$

A general-purpose library should not imply that one representation can preserve all properties of both.

## Selected trade-off

For runtime-selected metric sets, the design accepts one dynamic boundary.

The concrete measurement and mapping implementations remain statically typed within each metric. Only the heterogeneous collection requires a common runtime representation.

Conceptually:

$$
\text{runtime configuration}
\longrightarrow
\left[
\exists E_1.\operatorname{Eval}(E_1),
\ldots,
\exists E_n.\operatorname{Eval}(E_n)
\right]
$$

This provides:

* arbitrary runtime metric subsets;
* evaluation of selected metrics only;
* one stable and storable score-set type;
* linear growth with the number of selected metrics;
* no exponential enumeration of configurations;
* no runtime compiler.

The corresponding cost is one dynamic dispatch operation per selected metric.

This is not assumed to be universally optimal. It is the smallest general mechanism that preserves runtime heterogeneity without transferring the cost into exponential code generation, inactive computation, or JIT infrastructure.

## Application-level specialization

Applications whose metric composition is known at compile time may use a concrete static representation instead.

Applications whose metric composition is selected at runtime may use the dynamic representation.

An application may also use both, selecting a static representation for common predefined configurations and a dynamic representation for runtime overrides.

This is an application-level optimization rather than an intrinsic two-path requirement of the library.

The general principle is:

$$
\text{compile-time information}
\longrightarrow
\text{type-level composition}
$$

$$
\text{runtime information}
\longrightarrow
\text{value-level composition}
$$

The design therefore does not attempt to eliminate all runtime dispatch. It limits runtime dispatch to the point where runtime-selected heterogeneity must be represented, while preserving concrete generic types below that boundary.
