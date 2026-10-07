# Hydra 0.1 Execution Model

Status: **IN PROGRESS**.

Hydra 0.1 executes typed HIR with an interpreter. Function arguments are evaluated left-to-right. Each function call receives a fresh local frame. The call depth is limited to 512 frames and execution is protected by a deterministic step budget to prevent accidental unbounded execution in the reference interpreter.

Integer arithmetic uses checked 64-bit operations. Runtime failures become E4xxx diagnostics associated with the originating HIR span.

