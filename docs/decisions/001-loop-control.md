# D001: Value-less loop control for `while`

- Status: **Accepted**
- Date: 2026-10-08
- Compatibility: **intentional breaking change** (two new reserved keywords; otherwise an additive control-flow extension)
- Supersedes: none
- Selected family: control-flow extension (`break` / `continue`) only
- Human approval: **GRANTED for normative specification on 2026-10-08**, after independent architectural review of the lexer, parser, AST, resolver, checker, `Never`, HIR, runtime `Flow`, state machine, execution budget, versioning, and diagnostics. Implementation requires separate authorization.

## Problem

Hydra 0.1 has a statement `while`, `return`, typed structured HIR, and runtime `Flow::Value` / `Flow::Return`, but no way to exit or restart a loop early. The checker currently reduces unconditional statement non-completion to `stmt_diverges` and a boolean `terminated`, and uses `Type::Never` for no-normal-value branches. That information cannot tell a containing `while` whether to consume a break, consume a continue, or propagate a function return. This decision specifies the smallest first extension without inventing collections, value-producing loops, or a generic effect framework.

## Alternatives

- Reserve `break` / `continue` as bare statements **(selected)**.
- Parse them as contextual keywords while retaining 0.1 identifiers. Rejected for the first version: a bare `break` or `continue` is currently also a valid name expression or tail value; it cannot be both a statement and an expression in the same position without a more complex syntactic disambiguation or different syntax. Contextual handling alone does not guarantee compatibility.
- Break-with-value / expression `loop`, labels, named loops. Rejected: would require extra typing, target identity and syntax decisions unnecessary for current `while`.
- Lower loop control as returns, magic builtins, strings, or integer sentinels. Rejected: changes semantic identity and incorrectly conflates function and loop exits.
- Convert to MIR/CFG/SSA or introduce a generalized effect system first. Rejected: structured HIR/interpreter already supply a sufficient consumer and producer boundary.
- Have the resolver own loop-depth rejection. Plausible because it visits lexical scopes, but rejected: effect validity and typing both live in the checker; splitting semantics would duplicate control-context state and diagnostics ownership.

## Chosen semantics

### Syntax and placement

Add **reserved keyword** tokens `break`, `continue`. Each is a **value-less statement**:

```text
block-entry := let-stmt | while-stmt | return-stmt | break-stmt | continue-stmt | expression
break-stmt := "break"
continue-stmt := "continue"
```

They follow exactly the existing `return`-statement newline/`}` boundary policy; semicolons remain prohibited. There are no labels, loop names, arguments, optional values, or expression-position variants. `break 3`, `continue 3`, `break(label)`, and `return break` are syntax errors (E1101); an identifier after either keyword cannot be parsed as part of that statement. A comment and then a line break follow the normal comment policy. `break`/`continue` cannot be used as bindings, parameters, function names, or other identifiers once reserved.

### Loop-context legality and targeting

`break` and `continue` are legal only within the **lexical body** of at least one enclosing `while` in the same function. `break` exits the **nearest** enclosing eligible `while`; `continue` skips the rest of that loop-body iteration and immediately begins the loop's next condition evaluation. Lexical braces and `if` expressions do not create new loop targets. A nested `while` shadows the enclosing target **only in its body**; its condition is analyzed at the outer loop depth. Thus a `break` written in an inner loop's condition (e.g. inside a block expression) targets an outer loop, if present, and is illegal if no outer loop exists. Inside the inner body it targets the inner loop.

Each function starts with loop depth zero. A call to another function never transmits a caller's loop context. The compiler must reject out-of-loop occurrences even in statically unreachable source or dead branches: existing Hydra policy type-checks unreachable source.

The **checker alone** owns these source-level placement diagnostics, while the parser accepts syntactically well-formed loop-control statements and the resolver traverses them without another legality check. Enter loop context only while checking the `while` body, after checking its condition; always restore it on exit. AST/HIR preserve the source spans on the statements.

### Effects, blocks, types and reachability

`break` and `continue` are statements with **no value and no expression type**. For path analysis they terminate the current block's normal continuation, but they are distinct effects: `Break` and `Continue`. `return` likewise remains a function effect, and execution that cannot yield a value (e.g. a call statically returning `Never`) has an independent non-normal outcome. Runtime errors remain a `Result::Err` and are never an expression value.

Use a **bounded structured checker effect summary**, conceptually:

```text
Outcome = {
    falls_through: bool,     // at least one path supplies a normal continuation
    returns: bool,           // some path exits the current function
    breaks: bool,            // some path exits the active lexical loop
    continues: bool,         // some path restarts the active lexical loop
    may_diverge: bool,       // some path has no normal continuation without these effects
}
```

This is a description of semantics, **not a mandated Rust API**. The summary must attach to structural checking of blocks/expressions; it need not be publicly embedded in HIR. Boolean possibility flags are sufficient for this first unlabelled `while` family if nested loop effects are consumed at each body boundary, with the inner loop's condition analyzed in the *outer* context. No generic first-class effect algebra is required. Any `Never`-typed call with unknown non-completion can conservatively mark `may_diverge`.

Composition rule: `sequence(A,B).falls_through = A.falls_through && B.falls_through`. Propagate A's non-normal effects, plus B's effects **only along A's fall-through paths**; check B for type errors even if it is unreachable. For `if`, first account for effects of the condition, then merge only reachable branches (missing `else` contributes a normal `Unit` branch). For strict unary/binary, assignment RHS, argument lists and return operands, evaluate left to right, abort pending operations on a non-normal effect, and combine path outcomes accordingly. `&&` and `||` must preserve their skipped-RHS normal branch; a right operand that transfers control does **not** imply the entire boolean expression never returns normally. A type mismatch in unreachable source is still diagnosed; unreachable statements do not create reachable effects.

`Type::Never` continues to mean that **no normal value is produced**; it does not identify *why*. For example, a block ending in unconditional break has no normal value and may be given block type `Never` for existing bottom-type compatibility, but the *effect summary* must still say `breaks`, not `returns` or `may_diverge`. An `if` with a normal `Int` branch and a break-only branch has type `Int` for its normal path; an `if` where all branches transfer control is `Never` for its expression value and carries the distinct transferred effects. `Type::join(Never,T)` continues to yield `T` for the **normal-value type**; join alone never discards the corresponding branch effect. For a `return` whose operand first breaks/continues, propagate that loop effect **without emitting Return**.

**Statement `while` retains normal type `Unit`**, without a break value. It may complete normally when the condition evaluates to `false` or the body breaks. Do **not** treat a body with break/continue or return effects as proof that the entire while statement has type `Never`; iteration may be skipped. Its effect summary consumes body `breaks`/`continues`, propagates reachable `returns`/`may_diverge`, and models a normal exit conservatively, except if evaluating the condition itself is unconditionally non-normal. If the **condition** carries an effect targeted to an *outer* loop, propagate it unchanged; it is not consumed by this while. No constant-true/nontermination proof is required in the first version. The type of a containing block or tail expression follows its possible normal paths, not the mere presence of any terminal statement.

### Runtime flow, HIR, and execution order

Add dedicated `Stmt::Break` / `Stmt::Continue` and `HirStmt::Break` / `HirStmt::Continue` variants with spans. No `Type` variants, HIR expression variants, builtin dispatch, generated return, or magic sentinel is needed. Conceptually extend interpreter control to `Flow::{Value(Value), Return(Value), Break, Continue}`. Executing the statement produces its respective effect. `eval_block`, `if`, nested expression blocks, unary/binary evaluation, assignments, and call *argument* evaluation propagate it just as they now propagate `Return`, respecting conditional/short-circuit evaluation. Effects in the called function itself cannot escape its own function boundary.

A `while` handles normal body completion by repeating, consumes body `Continue` by re-evaluating **its condition**, consumes body `Break` by completing with `Value(Unit)`, and propagates `Return` out of its function body. A control effect emitted during evaluation of the **condition** is not owned by that while and propagates to an enclosing loop/function boundary. At `call_function`/entry completion, an uncaught Break/Continue is an invalid-typed-HIR **E9004** diagnostic, never `Value(Unit)` or a `Return`. Valid source is prevented from reaching this state by checker legality and scoping rules. A runtime internal error is not a source-level diagnostic substitute.

Preserve existing left-to-right argument evaluation, strict operator order, `&&`/`||` short-circuit, and existing `return` semantics. The current interpreter ticks at statement/expression boundaries and a loop-body iteration boundary. The eventual runtime change must ensure that the **same per-iteration boundary tick is incurred when the body completes via `continue`**, before the next condition evaluation, so an unconditional `continue` cannot avoid fuel consumption or change the step-budget accounting for ordinary iteration. `break` can exit without another iteration charge. No new execution budget limit is introduced.

### Diagnostics and ownership

- Parser: malformed `break`/`continue` forms or missing newline/`}` use existing **E1101** and standard parser progress/recovery rules.
- Resolver: no duplicate loop-control diagnostic; traverses applicable AST but keeps lexical name-binding responsibility.
- Checker: **E3010** means `break` outside an eligible enclosing `while` body; **E3011** means `continue` outside an eligible enclosing `while` body. These IDs are reserved by this acceptance in `docs/ERROR_CODES.md`. Attach the offending keyword as the primary span; do not reuse E3006 etc.
- Interpreter: malformed internal HIR whose control effect escapes a function boundary reports existing **E9004** with its originating keyword span (runtime flow may need a span, or carry it to the boundary). Valid source must never produce this diagnostic.

## Adversarial cases and exact results

For readability the snippets use Hydra's **newline delimiters**, not semicolons; helper names denote appropriately declared functions or variables.

| Shape | Result |
| --- | --- |
| `while c { if p { break }\n work() }` | If `p` true, exit loop; otherwise call `work`; conditional break does not prohibit later reachable statements. |
| `while outer { while inner { break }\n continue }` | Inner break exits inner loop only; outer continue restarts outer condition. |
| `while c { if stop { return 5 }\n if skip { continue }\n work() }` in an `Int` function | `stop` returns from function; `skip` restarts loop; otherwise `work` executes. |
| `fn f() { break }` or `fn f() { continue }` | Checker source error even when unreachable. |
| `while c { if p { break } else { continue }\n work() }` | `work` is checked but never executes from that `if`; effects remain distinct. |
| `while c { let x = { break }\n work() }` | No binding/write; the nearest loop exits. Block expression's no-normal-value type does not mask Break. |
| `while c { return { continue } }` | Continue is consumed by the loop; function does not return. |
| `while outer { while { break } { work() }\n work() }` | The inner condition breaks **outer** loop; inner `while` must not consume it. |
| `while { break } { work() }` with no outer loop | Static error; a loop's own condition has no loop target. |
| `while c { print({ continue }) }` | `print` does not run; continue restarts loop (argument evaluated first). |
| `while c { false && { break } }` | RHS is skipped; no Break. `Never` RHS does not make the `&&` expression unconditionally `Never`. |
| `while c { if p { break } else { return } }` | Break exits loop, Return exits function; both branches lack a value, with distinct effects. |
| `while c { f() }` where `f` has an uncontained `break` | `f` fails static checking; it cannot escape the call to target caller's while. |

Specific traps falsified by the design: combining all effects into `Never`; one accumulated block `terminated` flag; consuming condition effects as inner-body break; returning a value from a break-only block; dropping break/continue in strict argument evaluation; bypassing budget charge on continue; accepting keyword-using old identifiers while claiming unconditional backwards compatibility.

## Compiler impact and implementation obligations (future campaign only)

- Lexer/grammar: add keyword tokens and classify old identifiers; preserve newline and syntax-depth recovery behavior.
- AST/parser: parse only two statement forms with spans; reject malformed operands; handle `}`-adjacent forms.
- Resolver: add exhaustive traversal cases; preserve per-function lexical scopes; no duplicate placement validation.
- Type system/checker: track lexical loop body context and the minimal sound effect summary across statements, expressions, `if`, strict vs short-circuit evaluation, and nested while; preserve unreachable-code checking; avoid a global `terminated` bool that confuses effects.
- Typed HIR: add explicit `Break`/`Continue` statement variants and maintain typed expression semantics.
- Runtime/backends: distinguish effects in `Flow`; propagate across blocks/expressions; consume only in the owning while body; prevent escape through function return; tick on continue; use E9004 for malformed internal HIR.
- Diagnostics: implement the reserved E3010/E3011 source placement errors and tests for their spans; keep E1101/E9004 meanings intact.
- Documentation: only after acceptance, update the normative specs below and release/compatibility ledger as required by versioning policy.

## Testing obligations (implementation campaign, NOT now)

Positive: loop exit and repeat; nested blocks, `if` true/false branches, direct before-`}`, and calls with block-expression arguments. Negative: outside loop in function/tail expression, inner condition with no outer loop, invalid attached operands/labels, reserved old identifier spellings, and type errors after unreachable statements. Nested: independent function contexts, multiple loops, condition vs body, and conditionally mixed return/break/continue. `Never`: bottom-type branch joins, absent else, assignments, strict arguments, short-circuit, `return` operands, and never-returning calls. Runtime: exact side effects/evaluation order, no execution after effect, correct `Unit` loop exit, infinite `continue` must exhaust E4006, and mutated HIR escaping effects must report E9004 with the source span. Properties: deterministic generated nesting ensures every well-typed loop effect is consumed in its correct body and never crosses a function boundary; valid spans, no invalid HIR success, bounded evaluation. Fuzz: existing lexer/parser/compile/runtime harnesses are reusable, but keyword seeds, nesting and effect mutation inputs must be measured and extended as necessary when implementation begins. Keep 0.1 corpus and platform/MSRV gates green.

## Compatibility impact

**Intentional breaking change**, as defined by `docs/VERSIONING.md`: in 0.1 both `break` and `continue` are ordinary `TokenKind::Identifier`, usable as local/parameter/function names and expression references. Keyword reservation turns otherwise valid 0.1 programs into syntax errors and changes their diagnostic from success to E1101, or changes an invalid-usage diagnostic depending on context. These precise name collisions must be included in the future breaking-change ledger/tests. No other 0.1-valid program is intended to become invalid, change a runtime value/effect, or change its diagnostic. The accepted 0.1 tests remain the baseline for this guarantee. Valid new loop-control statements are additions; outside-loop statements will receive new source diagnostics. There is no published Hydra release or tag whose compatibility is silently being changed.

## Normative specifications locked by acceptance

The 2026-10-08 human acceptance authorizes the following documentation-only normative updates. They describe Hydra 0.2's **accepted, unimplemented** contract, alongside the preserved implemented Hydra 0.1 contract:

1. `spec/GRAMMAR.md`: `block-entry`, keyword list, `break-stmt`/`continue-stmt`, newline/brace boundary, operand/label prohibition, and reserved-identifier compatibility note.
2. `spec/TYPE_SYSTEM.md`: placement rules, nearest-loop and inner-condition targeting, `while`'s `Unit` completion, `Never` as no-normal-value bottom type **separate from loop/function effects**, path-sensitive reachable-effect checking and unchanged unreachable-type-check policy; conditional and strict/short-circuit cases.
3. `spec/EXECUTION_MODEL.md`: propagation/consumption of runtime Break/Continue, nested blocks/expressions/calls, condition vs body ownership, function barrier, E9004 malformed-HIR invariant and E4006 tick-on-continue requirements.
4. `docs/ERROR_CODES.md`: E3010/E3011 reservations with keyword spans; parser E1101 vs internal E9004 boundaries.
5. `HYDRA_0_2_D001_ACCEPTANCE.md`: record the intentionally breaking keyword reservation without rewriting the historical Hydra 0.1 baseline. Corpus/regression extensions are required in the separately authorized implementation campaign.

**Accepted after independent architectural review; normative contract locked; not implemented.** Acceptance neither authorizes Rust changes nor begins the D001 implementation campaign. H15 Collections remains deferred.
