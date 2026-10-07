# Hydra 0.1 Execution Model

Status: **IMPLEMENTED for the Hydra 0.1 core subset**.

Hydra 0.1 executes typed HIR with an interpreter. Function arguments are evaluated left-to-right. Each function call receives a fresh local frame. The call depth is limited to 128 frames and execution is protected by a deterministic step budget to prevent accidental unbounded execution in the reference interpreter. The conservative depth limit is part of Hydra's runtime contract: pathological recursion fails with a structured runtime diagnostic before exhausting the host stack.

Integer arithmetic uses checked 64-bit operations. Runtime failures become E4xxx diagnostics associated with the originating HIR span.

Execution begins at `main`, which must exist and take no parameters. Blocks and calls propagate `return` explicitly through interpreter control flow. Boolean `&&` and `||` short-circuit. The interpreter consumes typed HIR only; it does not perform source-level name lookup or type checking at runtime.
