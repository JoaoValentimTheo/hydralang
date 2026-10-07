# Hydra 0.1 Execution Model

Status: **IMPLEMENTED for the Hydra 0.1 core subset**.

Hydra 0.1 executes typed HIR with an interpreter. Function arguments are evaluated left-to-right. Each function call receives a fresh local frame. The call depth is limited to 128 frames; exceeding it reports E4003. Each run starts with a deterministic budget of 1,000,000 evaluation steps; exhausting it reports E4006. These guards are part of the 0.1 reference-interpreter contract and prevent pathological recursion or execution from consuming the host indefinitely.

Integer arithmetic uses checked 64-bit operations. Overflow reports E4004 and integer division/remainder by zero reports E4005. Floating-point arithmetic uses Rust `f64` operations, including IEEE infinities and NaN rather than integer-style divide-by-zero diagnostics. Runtime failures become E4xxx diagnostics associated with the originating HIR span.

Execution begins at `main`, which must exist and take no parameters. Blocks and calls propagate `return` explicitly through interpreter control flow. Boolean `&&` and `||` short-circuit. The interpreter consumes typed HIR only; it does not perform source-level name lookup or type checking at runtime.

`print` writes its single argument without a trailing newline; `println` writes the same representation followed by a newline. `Unit` is rendered as `()`. Diverging `Never` expressions have no runtime value: if evaluation of a builtin or function argument returns from the current function, the pending call is not executed.

Runtime never executes statements after a propagated `return`. The compiler may still have resolved and type-checked such source before HIR execution, as specified by the 0.1 unreachable-code policy.
