# Hydra 0.1 Execution Model

Status: **IMPLEMENTED for the Hydra 0.1 core subset**.

Hydra 0.1 executes typed HIR with an interpreter. Function arguments are evaluated left-to-right. Each function call receives a fresh local frame. The call depth is limited to 128 frames; exceeding it reports E4003. Each run starts with a deterministic budget of 1,000,000 evaluation steps; exhausting it reports E4006. These guards are part of the 0.1 reference-interpreter contract and prevent pathological recursion or execution from consuming the host indefinitely.

Integer arithmetic uses checked 64-bit operations. Overflow reports E4004 and integer division/remainder by zero reports E4005. Floating-point arithmetic uses Rust `f64` operations, including IEEE infinities and NaN rather than integer-style divide-by-zero diagnostics. Runtime failures become E4xxx diagnostics associated with the originating HIR span.

Execution begins at `main`, which must exist and take no parameters. Blocks and calls propagate `return` explicitly through interpreter control flow. Boolean `&&` and `||` short-circuit. The interpreter consumes typed HIR only; it does not perform source-level name lookup or type checking at runtime.

`print` writes its single argument without a trailing newline; `println` writes the same representation followed by a newline. `Unit` is rendered as `()`. Diverging `Never` expressions have no runtime value: if evaluation of a builtin or function argument returns from the current function, the pending call is not executed.

Runtime never executes statements after a propagated `return`. The compiler may still have resolved and type-checked such source before HIR execution, as specified by the 0.1 unreachable-code policy.

## Hydra 0.2 accepted loop-control execution — implementation pending

This is the accepted D001 **normative execution contract**, not a claim that Hydra 0.1 currently implements `break` or `continue`. The conceptual interpreter channel expands to:

```text
Flow::Value(Value)
Flow::Return(Value)
Flow::Break
Flow::Continue
```

The precise Rust representation is not fixed by this contract. Dedicated AST/typed-HIR statements with originating keyword spans generate `Break` and `Continue`. Blocks, `if`, block expressions, strict unary/binary operands, assignments, and user-function/builtin argument evaluation propagate those flows without executing a pending operation. The interpreter preserves left-to-right evaluation and boolean `&&`/`||` short-circuit order. `Return` exits the current function; runtime failures remain `Result::Err` rather than an ordinary `Flow` value.

`while` establishes control ownership **only for its body**. A body `Break` exits the nearest such `while` with normal `Unit` completion; a body `Continue` skips the remaining body and starts the next condition evaluation; a normally completed body repeats. A body `Return` propagates to the function. During evaluation of a `while` **condition**, this loop does not consume `Break` or `Continue`: any such effect propagates to a containing outer `while` (if valid), and the current condition/body operation stops. A false condition permits normal `Unit` completion. Nested loops each consume only control from their own bodies.

Every function call creates a loop-control boundary: valid source cannot transfer a `Break` or `Continue` from inside a called function to the caller's loop. If malformed typed HIR lets either effect escape a function or entry boundary, the interpreter must diagnose the originating keyword span as **internal E9004**. This is an invariant failure, not a source placement diagnostic (E3010/E3011). The entry and call boundaries must not silently convert escaping effects to `Unit` or `Return`.

The deterministic **1,000,000-step** reference-interpreter budget stays in force. The existing per-iteration boundary `tick` must be charged even when a loop body reaches `continue`, **before the next evaluation of the condition**. Therefore `while true { continue }` must exhaust fuel with E4006 rather than evade the limit. `break` may exit without an additional iteration charge. No new budget limit is introduced; this accepts a future execution path under the existing reference-interpreter resource policy.
