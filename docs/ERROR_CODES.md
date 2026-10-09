# Error Codes

Hydra diagnostics use stable phase-oriented families. Codes are not reused for unrelated failures.

| Code | Phase | Meaning |
| --- | --- | --- |
| E1001 | Lexer | Unexpected character |
| E1002 | Lexer | Unterminated string literal |
| E1003 | Lexer | Unknown string escape |
| E1101 | Parser | Invalid or incomplete syntax |
| E1102 | Parser | Invalid or out-of-range integer literal |
| E1103 | Parser | Invalid floating-point literal |
| E1104 | Parser | Invalid assignment target |
| E1105 | Parser | Maximum syntax nesting depth exceeded |
| E1106 | Parser | Maximum expression-tree depth exceeded |
| E2001 | Resolution | Duplicate function declaration |
| E2002 | Resolution | Duplicate local declaration or parameter in one scope |
| E2004 | Resolution | Undefined name |
| E2005 | Resolution | Assignment to an immutable binding |
| E3001 | Type checking | Unknown type name |
| E3002 | Type checking | Type mismatch, including invalid return type |
| E3003 | Type checking | Invalid unary or binary operand types |
| E3004 | Type checking | Incompatible `if` branch types |
| E3005 | Type checking | Value-producing `if` without `else` |
| E3006 | Type checking | First-class function value unsupported in 0.1 |
| E3007 | Type checking | Unsupported builtin argument type |
| E3008 | Type checking | Calling a local value unsupported in 0.1 |
| E3009 | Type checking | Function or builtin arity mismatch |
| E3010 | Type checking | `break` outside an eligible enclosing `while` body (D001 implemented) |
| E3011 | Type checking | `continue` outside an eligible enclosing `while` body (D001 implemented) |
| E3012 | Type checking | D002 constant positional projection on a normally valued non-tuple |
| E3013 | Type checking | D002 constant positional projection index outside the tuple's static arity |
| E3014 | Type checking | D002 inferred structural tuple-type nesting exceeds 64 layers (2026-10-08 Option A) |
| E3015 | Type checking (D003 reserved, not emitted) | Empty List literal without a permitted exact expected `List<T>` |
| E3016 | Type checking (D003 reserved, not emitted) | Inferred aggregate type containing List exceeds combined List/Tuple depth 64 |
| E3017 | Type checking (D003 reserved, not emitted) | Syntactically valid builtin List type with other than one type argument |
| E3018 | Type checking (D003 reserved, not emitted) | Normally valued non-List base of index expression |
| E3019 | Type checking (D003 reserved, not emitted) | Normally valued index expression with non-Int type |
| E4001 | Runtime | Missing `main` function |
| E4002 | Runtime | `main` declares parameters |
| E4003 | Runtime | Maximum call depth exceeded |
| E4004 | Runtime | Checked integer overflow |
| E4005 | Runtime | Integer division or remainder by zero |
| E4006 | Runtime | Deterministic execution-step budget exhausted |
| E4007 | Runtime (D003 reserved, not emitted) | List index outside `0 <= index < length`, including negative or empty-List index |
| E9001 | Internal | Function ID space exhausted during resolution |
| E9002 | Internal | Resolver scope invariant violated |
| E9003 | Internal | Type-checker/HIR lowering invariant violated |
| E9004 | Internal | Typed-HIR/runtime invariant violated |
| E9005 | CLI | Compiler returned neither HIR nor diagnostics |

E9xxx diagnostics represent compiler invariants rather than ordinary user-program failures. Ordinary malformed source should terminate with an E1xxx-E4xxx diagnostic and must not require an E9xxx path.

The **implemented and frozen** D001 loop-control checker owns E3010 and E3011 for placement violations, with the offending keyword as the primary span. Malformed `break`/`continue` syntax remains E1101; a loop-control effect escaping a function from malformed typed HIR remains internal E9004. This extends the historical 0.1-only diagnostics above.

The accepted D002 tuple contract assigns E3012 and E3013 exclusively to the **type checker**, never the lexer, resolver or runtime. Both point primarily from the projection dot through the numeric index. E3012 requires a normally valued non-tuple base. E3013 requires a normally valued tuple base with a statically out-of-range zero-based index, and its message must identify the index and tuple arity. A base whose whole-expression normal type is `Never` propagates its D001 effect rather than triggering E3012/E3013. These checks are implemented and frozen under D002.

D002 retains **E1101** for tuple syntax errors, missing elements/delimiters, malformed projection suffixes and 65-or-more-field tuples (source tuple span; maximum 64). **E1102** rejects a projection index that exceeds its accepted unsigned integer representation without truncation. **E1104** rejects a projected-field assignment target; **E1105** enforces both the unchanged 128-level recursive syntax guard and the 64-level **written tuple-type** nesting guard, with appropriate source spans; **E1106** applies the unchanged 256-depth expression-tree guard to tuples and projection bases. Existing **E3002** covers structurally mismatched tuple types, **E3003** invalid tuple operators/equality operand typing, **E3004** incompatible `if` result types, and **E3007** whole-tuple arguments to primitive-only `print`/`println`. **E9003** remains an internal checked-HIR lowering invariant; **E9004** covers malformed runtime typed-HIR tuple/projection state (invalid base, bounds, inconsistent value/type, corrupt aggregate) without panics. Iterative tuple equality spends the existing runtime fuel, with **E4006** on exhaustion.

**2026-10-08 human-approved D002 Option A amendment:** The 64-layer structural tuple-type limit applies equally to written annotations and inferred types. The parser retains E1105 for written type nesting above 64. The **checker** emits **E3014** when an expression first constructs an inferred type with tuple depth above 64, with that expression as primary span and both the permitted and actual nesting depths in its message. Primitive depth is zero; tuple depth is one plus the greatest child depth; grouping adds zero. Every checked program must satisfy this before typed HIR can execute, including locals, function arguments/returns, joins and projections. Whole-expression `Never` retains the frozen D001 effect semantics. Internal malformed HIR alone retains defensive E9004. This targeted diagnostic is the sole new ownership introduced by the amendment.

Diagnostics originating from source syntax or HIR evaluation carry a primary source span. Spans must remain within the originating UTF-8 source and on character boundaries. E4001 uses the first function span when one exists, or the empty start-of-source span for an empty program; E4002 points at `main`; call-depth, arithmetic, and step-budget failures point at the originating HIR expression/call span.

Two deliberate edge policies are worth making explicit: a semicolon reports E1001 because it is not a 0.1 token, and direct `-9223372036854775808` reports E1102 because the positive literal token is parsed before unary negation and does not fit in `Int`.

## D003 diagnostic reservation — accepted 2026-10-09, NOT IMPLEMENTED

The full pre-D003 registry above had **no collision** for E3015–E3019 or E4007. Those six codes are reserved **normatively for future D003 implementation**; their presence in this registry does **not** imply any current emitter exists. E3015 applies only when `[]` has no expected `List<T>` from the five accepted direct contexts; E3016 applies to inferred combined depth >64 **containing a List**, with constructing expression span and actual/maximum depth; E3017 checks syntactically well-formed `List` type-argument arity; E3018 and E3019 check only normally completing index operands. Non-normal operands preserve D001 effects. E4007 is an **ordinary source-reachable** bounds failure at the index operation, never internal E9004.

Existing codes retain phase ownership: **E1101** malformed brackets, invalid List/type punctuation, excess 256 literal elements; **E1104** assignment through an index; **E1105** written combined aggregate depth 65 and recursive syntax nesting 128; **E1106** expression-tree depth 256; **E3001** unknown type; **E3002** heterogeneous List elements/incompatible structural types; **E3003** disallowed List operators; **E3004** incompatible branch result types; **E3007** whole-List print/println; **E3014** tuple-only inferred depth 65; **E4006** existing fuel exhaustion; **E9004** malformed internal typed HIR or value/type invariants. **E3009** remains function/builtin *value-call* arity only, not `List<T>` arity. Syntactically complete wrong List arity reaches the checker once; malformed type syntax is diagnosed by the parser once. No duplicated cross-phase diagnostics. Every public diagnostic has an exact source span that respects UTF-8 scalar boundaries. See `HYDRA_0_2_D003_ACCEPTANCE.md` and `spec/TYPE_SYSTEM.md`.
