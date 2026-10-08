# Hydra 0.2 design gate: first feature family

- Gate date: 2026-10-08
- Starting authoritative `main` / `origin/main`: `9ed6d90f503d7e1812de2e028d8a6c2806a5d9c8`
- Starting worktree: clean, after `git fetch origin`; no tags or releases
- Starting CI reference: GitHub Actions run `37803469718` (the Hydra 0.1 platform-migration gate)
- Status: **DESIGN COMPLETE; IMPLEMENTATION NOT STARTED**
- Exactly one first family selected: **control-flow extension (`break` / `continue`)**
- Decision record: [`docs/decisions/001-loop-control.md`](docs/decisions/001-loop-control.md), **Proposed — human approval required**
- Hydra 0.1 foundation, post-foundation audit, baseline freeze, and CI migration: complete; H15 Collections: **DEFERRED**

This document is a design result, not a specification amendment, implementation approval, or release announcement. The accepted Hydra 0.1 contract remains in `spec/`.

## Architectural evidence and dependency graph

The inspected chain is `hydra-lexer` → `hydra-parser` / `hydra-ast` → `hydra-resolve` → `hydra-check` / `hydra-types` → `hydra-hir` → `hydra-runtime`, with `hydra-stdlib` providing stable builtin identities. These are specific extension points in the current code:

- `crates/hydra-lexer/src/lib.rs`: keyword recognition is an exhaustive identifier spelling match; `break` and `continue` are currently ordinary identifiers.
- `crates/hydra-parser/src/lib.rs`: `parse_block_inner` recognizes statement-start `let`, `while`, and `return`; `parse_while` parses a condition *before* the body block; `require_line_boundary` enforces newline/closing-brace boundaries. Type syntax is currently just an identifier.
- `crates/hydra-ast/src/lib.rs`: `Stmt::{While,Return}` already carry spans; `ExprKind::{If,Block}` admit nested control effects without new expression grammar.
- `crates/hydra-resolve/src/lib.rs`: `resolve_stmt` traverses `While` condition and body, lexical scopes are per function, and IDs are program-local; no cross-module namespace machinery exists.
- `crates/hydra-types/src/lib.rs`: `Type` has only `Int`, `Float`, `Bool`, `String`, `Unit`, `Never`; `join(Never,T)=T`; no parametric, tuple, nominal, or constant-length types exist.
- `crates/hydra-check/src/lib.rs`: `check_block` uses a single accumulated `terminated` boolean and `stmt_diverges`; `check_stmt` already checks `While` and `Return`; `check_expr` handles `If`, `Block`, strict evaluation, and short-circuit boolean operators. `Never` is currently asked to summarize non-normal completion.
- `crates/hydra-hir/src/lib.rs`: structured `HirStmt::{While,Return}` and typed `HirExpr` already preserve control constructs; no MIR/SSA is required for an interpreter-local loop effect.
- `crates/hydra-runtime/src/lib.rs`: `Flow::{Value,Return}` propagates through blocks, `if`, operators, assignment, argument evaluation, and `while`; `while` has a natural consumer for two additional flow cases. `tick` is charged on statements, expressions, and the per-body loop boundary; E4006 enforces the 1,000,000-step budget.
- `crates/hydra-stdlib/src/lib.rs`: only `print` / `println`; `break`/`continue` are control statements and require no builtin API.

Graph notation: **H** = hard semantic prerequisite, **C** = convenient future reuse, **→** = a dependency or later design choice. Hard means necessary **for the candidate as named**, not necessarily for a different simplified candidate.

```text
Already implemented: lexer → block parser/AST → lexical resolver → checker → typed HIR
                     → reference interpreter; statement while + function return + Never

Control flow → [H: loop-local control-effect checking, within this family]
             → [C: later CFG/MIR lowering, no requirement today]

Type-constructor foundation → [H: type argument grammar/arity, representation,
                                   name lookup, builtin vs user-defined policy]

List<T>    → [H: type-constructor decision, homogeneous element semantics]
           → [C: iteration protocol, generics for user functions]
Tuple      → [H: tuple type/arity/identity and literal grammar decisions]
           → [C: destructuring/patterns; generic type-constructor syntax is optional]
Set<T>     → [H: type-constructor decision, equality/hashability or alternate
                membership semantics, delimiter distinction from blocks]
           → [C: iteration protocol, user-defined equality/hashing]
Array<T,N> → [H: decision to include arrays, element type + length-in-type
                representation, constant-length grammar/evaluation]
           → [C: generic const parameters, optimized layout]

Modules/imports → [H: compilation units, namespace/visibility, import cycle policy,
                    cross-unit IDs and CLI/source-loading strategy]
Algebraic data types → [H: nominal type/constructor identity, declaration grammar,
                         field/variant semantics]
                     → [C: modules, pattern matching, generic ADTs]
Generics → [H: parameterized type/function syntax and identity, substitution,
             inference/instantiation policy]
         → [C: ADTs, modules, constraints/traits, monomorphizing backend]
```

In particular, **Tuple does not inherently require generics**, **List<T> does not inherently require user-defined generics**, and **module support is not inherently required for nominal ADTs**. Conversely, introducing `List<T>` as written requires deciding how a type argument is represented; a plain `TypeExpr { name }` and six-variant `Type` cannot represent it without such a decision. `Array<T,N>` need not commit to a general const-generics language, but its constant length must be a specified part of type identity.

## Per-candidate impact and semantic prerequisites

`L/P/A/R/T/H/X/D` refer to lexer, parser, AST, resolver, type checker/types, HIR, runtime, diagnostics. `●` means meaningful impact; `◐` means a bounded/traversal impact; `—` means none expected. This is **architecture exposure**, not a count of commits or invented implementation estimates.

| Candidate | Hard semantic work before acceptance | Convenient, *not hard* dependencies | L | P | A | R | T | H | X | D |
| --- | --- | --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| Control-flow `break`/`continue` | lexical loop legality; accurate path effects; nearest-loop consumption | MIR/SSA, labels | ● | ◐ | ◐ | ◐ | ● | ◐ | ● | ◐ |
| Type-constructor foundation | parameter syntax/arity; constructor identity; resolution and equality; builtin/user policy | first concrete collection, generics | ◐ | ● | ● | ● | ● | ◐ | — | ● |
| `List<T>` | type constructor; literals; homogeneous equality, mutability, indexing, ownership | user generics, iteration protocol | ◐ | ● | ● | ◐ | ● | ● | ● | ● |
| Tuple | arity/type/literal design, `(x)`/`()` disambiguation, equality | generics, destructuring | ◐ | ● | ● | ◐ | ● | ● | ● | ● |
| `Set<T>` | type constructor; set-vs-block syntax; key equality/hashability; order | iterators, user-defined hashing | ◐ | ● | ● | ◐ | ● | ● | ● | ● |
| Fixed `Array<T,N>` (optional) | whether to include; const `N`; element/length type rules; indexing | generic const parameters, fixed layout backend | ◐ | ● | ● | ◐ | ● | ● | ● | ● |
| Modules/imports | files/compilation units; namespace, visibility, cycles, symbol identity | packages, resolver caching | ● | ● | ● | ● | ● | ● | ● | ● |
| Algebraic data types | nominal identity, declaration/constructor/member semantics | modules, patterns, generics | ● | ● | ● | ● | ● | ● | ● | ● |
| Generics | parameter/substitution/instantiation/inference semantics; type identity | traits, ADTs, monomorphization | ● | ● | ● | ● | ● | ● | ● | ● |

| Candidate | Dependency load | Semantic complexity | Implementation surface | Architectural leverage | Risk of premature abstraction / lock-in | Testing burden | Future blocking value |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Control-flow** | **Low**: local effect model only | Medium: nested paths, `Never` | Bounded: existing structured flow chain | High: reusable effect discipline | Low/medium: incorrect effect summary would be costly | Medium: nesting, reachability, fuel | High: essential loop completeness |
| Type constructors | Medium/high: type grammar + identity policy | High: application and arity rules | Broad: types, parser, checker | High for collections/ADTs | **High**: design without a consumer | High: nested types/arity | High, but can follow a concrete need |
| `List<T>` | High: constructor first | High: value/alias/equality/indexing | Broad end-to-end | High | High: early generic/ownership commitment | High: bounds, aliasing, size | Medium/high |
| Tuple | Medium: self-contained tuple types | Medium/high: singleton grammar, equality | Broad end-to-end | Medium/high | Medium: shape/structural identity | High: arity/nesting | Medium |
| `Set<T>` | High: constructor and membership contract | High: hashing/order/mutability | Broad end-to-end | Medium | **High**: hash/key policy lock-in | High: collision/determinism | Low/medium |
| `Array<T,N>` | High: include? const length | High: const/type/layout | Broad end-to-end | Medium | **High**: length/layout syntax lock-in | High: limits/bounds/size | Low/medium |
| Modules/imports | High: compiler/CLI unit model | High: names, cycles, visibility | Very broad across pipeline | Very high at scale | High: file/package model lock-in | Very high: graph/filesystem | Medium until multi-file needed |
| ADTs | High: nominal identity | High: variants, fields, equality | Very broad | Very high | High: type/namespace design lock-in | Very high | High, but not first |
| Generics | Very high: parametric typing + instantiation | Very high | Very broad | Very high | **Very high**: abstraction without users | Very high | High, but downstream |

Qualitative scales reflect the number and difficulty of **unresolved dependencies** and distinct compiler/runtime boundaries, not weighted arithmetic. Feature surface size alone did not decide the result.

## Selection and alternatives

**SELECTED: control-flow extension (`break` / `continue`).** Current `while` supplies its natural consumer, the typed HIR has statement variants, and interpreter `Flow` already distinguishes ordinary values from function returns. The necessary effect-summary correction can be designed and implemented entirely inside this family. No collection/type constructor, nominal type, modules, generics, MIR/SSA, or new backend is a hard dependency. The scope includes checker path effects and their regression obligations; treating that as an optional follow-up would be unsound.

Rejected *as first*, without rejecting their long-term value: the type-constructor foundation has no accepted consumer and would ossify syntax early; each collection needs additional independent semantics; Tuple is comparatively self-contained but still introduces a new value/type family; modules require a compilation-unit and namespace design; ADTs require nominal identity; generics should follow concrete parameterization pressure. No composite family was selected.

## Proposed semantic contract (approval pending)

The full and authoritative **proposal** is in decision 001. Its key choices are:

1. `break` and `continue` are value-less, unlabelled **statements**, each alone on a statement line or directly before `}`. They are keywords. `break x`, `continue x`, labels, and loop names are invalid syntax; they are never values or builtins.
2. Legal only within the **body** of the nearest lexically enclosing `while`. That new loop does not become the target while checking its own condition. A containing outer loop remains the target for control written in an inner `while` condition. An uncontained use is a **checker** error; the parser accepts the statement and the resolver only traverses it.
3. `break` exits the nearest target `while`; `continue` re-evaluates its condition. Nested blocks/`if`/strict expressions propagate the effects without executing intervening operations. Functions are control barriers; a called function cannot break its caller's loop. `return` continues to exit its function.
4. Existing statement-style `while` yields `Unit` on normal completion and never yields a break value. The body can terminate normally, break, continue, return, or fail at runtime. The loop can still have a normal exit when its condition is false. A condition that never completes normally retains its non-normal effect.
5. `Never` remains a *type* meaning no normal value on that path; it is **not** a synonym for `return`, `break`, `continue`, or nontermination. Branch type joins can still use `Never` as bottom, but checker path analysis must separately track `falls_through`, `returns`, `breaks`, `continues`, and potentially diverges. Runtime failure is a separate `Result::Err` outcome.
6. Composition is path sensitive at the structural level: later statements contribute effects only on earlier fall-through paths, though unreachable code is still checked; conditional branches join alternatives; strict operands propagate in evaluation order; `&&` / `||` preserve short-circuit paths. A `while` consumes break/continue from its **body**, propagates returns, and never consumes effects from its **condition** targeting an outer loop.
7. Dedicated AST/HIR statement variants preserve identity; runtime `Flow::Break` and `Flow::Continue` are created at statements, propagated through blocks/expressions, consumed by the appropriate `while` body, and diagnosed as **E9004** only if malformed internal HIR allows them to escape to a function boundary.
8. Runtime budget accounting must charge each repeated iteration on the `continue` path as on ordinary body completion. Existing statement/expression tick order and left-to-right call/argument evaluation stay unchanged; no execution path can repeatedly continue without consuming E4006 fuel.
9. Proposed new checker diagnostic conditions: break/continue outside a loop (separate new E3xxx codes to allocate on implementation); malformed operand/label forms use the existing E1101 parser family; escaped effects from invalid HIR use existing E9004 (the same **invariant** family, not a user error).

## Adversarial findings resolved at design time

- An inner `while` containing `break`, followed on the **next line** by `continue` inside the outer `while`: inner break is consumed by the inner loop; outer continue targets the outer loop. No label needed.
- `if stop { break }` in a loop body can fall through when `stop` is false; it must **not** mark all subsequent code unreachable. With both branches transferring control, their different kinds must remain distinguishable.
- `if stop { return value } else { continue }` has no normal value but carries two distinct effects. `Never` alone cannot specify which consumer handles which outcome.
- `return { break }` inside a loop first evaluates its operand; the break propagates to the loop, and **no** return is emitted. The same rule applies to function argument evaluation and assignment RHS blocks.
- `while { break } { ... }` has no loop body target for the break; outside another loop it is a static checker error. Within an outer loop, the break targets that **outer** loop, so inner-loop condition handling must propagate it.
- `break` / `continue` in a nested block expression within a loop are legal and propagate through surrounding `if`/operator evaluation without producing an expression value. At top level of any function body with no loop they are static errors.
- An apparent `Never` result from an effect-only block must not make a surrounding `while` permanently `Never` when the effect is a locally consumed break and the loop can finish with `Unit`.
- A user-defined function called inside a loop has its own independent loop-depth state. It cannot target the caller's loop.
- `continue` cannot skip the loop iteration's budget charge, and malformed HIR effects cannot leak through a function result as a value.

## Test obligations for a later implementation campaign

No tests or fuzz targets are added by this gate.

| Class | Required proof |
| --- | --- |
| Positive | `break`/`continue` directly in a `while`, inside a block or conditional, and at a brace-adjacent statement boundary; normal `while` result `Unit` |
| Negative | top-level/function-level effects, outside-loop nested blocks, `break 1`, `continue 1`, labels, assignments/function names using newly reserved words |
| Nested control | nearest-loop exit/restart, inner-loop condition vs body, independent functions, no caller-loop access, conditional return vs break/continue |
| Reachability | after unconditional break/continue/return and conditional effects; unreachable code still type-checked but not executed |
| `Never` | join of effect-only vs value branches; both branches effectful; short-circuit RHS; strict operands/arguments and return operands; function declared `Never` |
| Runtime | loop exit vs condition re-evaluation, left-to-right expression and call order, effect propagation through nested expression blocks, E4006 under infinite continue |
| Invariant | hand-built escaped control effects and non-boolean conditions in HIR return E9004, no accidental interpretation as `Unit`/return |
| Deterministic properties | generated nested scopes: no escaped well-typed effects; local consume or function return; determinism, termination within runtime budget and valid spans |
| Fuzz | existing lexer/parser/compile-pipeline/runtime targets will reach new code if corpus includes the keywords; inspect harness coverage and add seeds/target extensions as needed *during implementation*, never in this gate |
| Platforms | frozen 0.1 corpus, stable/MSRV/tests and Linux/Windows/macOS gates plus same-version differential semantics after implementation |

## Compatibility, gate state, and required follow-up

**Classification: intentional breaking change**, limited at source level to keyword reservation. `break` and `continue` currently lex as `Identifier`; a Hydra 0.1 program may legally use them as local/function/parameter names. Reserving them makes those programs syntactically invalid (typically E1101), so calling the whole proposal a compatible extension would be false. Code not using those identifier spellings retains its value/effect/type behavior and intended diagnostics; the implementation must prove this with the unchanged 0.1 corpus and targeted identifier regressions. `break` / `continue` statements themselves newly become valid in well-formed loops and produce new diagnostics outside them. No public release/tag currently promises compatibility; `docs/VERSIONING.md` still requires an accepted decision, spec update, tests, and full validation **in the implementation campaign**.

Normative spec files remain unchanged: decision 001 lists exact required updates to `spec/GRAMMAR.md`, `spec/TYPE_SYSTEM.md`, and `spec/EXECUTION_MODEL.md`; its diagnostic requirements must also be recorded in `docs/ERROR_CODES.md` if approved. The next gate is human review of **one Proposed decision**. **Hydra 0.2 implementation has NOT started**; the 0.1 baseline is untouched and H15 Collections stays deferred.
