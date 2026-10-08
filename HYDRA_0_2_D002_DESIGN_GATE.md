# Hydra 0.2 D002 — second-family design gate

- Gate date: 2026-10-08
- Starting `main` / `origin/main`: `764b901ec8a0f0febfb3624c392d27f14b631ff4` (baseline clean before creating the D002 proposal)
- Baseline GitHub Actions: run `37821337289`, **5/5** at the frozen D001 checkpoint
- Frozen D001: `docs/decisions/001-loop-control.md`, `HYDRA_0_2_D001_FREEZE.md`; production commit `f5bb9e6bd8b8befde6bdc79d290983b0ba136d7b`, independent audit tests `8ab976c7abaa61b1c2e32e753834150a15575664`, freeze checkpoint `764b901ec8a0f0febfb3624c392d27f14b631ff4`
- **Exactly one D002 family selected: structural Tuple.** Decision: [`docs/decisions/002-tuples.md`](docs/decisions/002-tuples.md), **PROPOSED**.
- **D002 IMPLEMENTATION NOT STARTED. Human approval required.** No accepted D002 normative contract or release exists.

## Current architecture and selection evidence

The post-D001 pipeline is `hydra-lexer` → `hydra-parser`/`hydra-ast` → `hydra-resolve` → `hydra-types`/`hydra-check` → typed `hydra-hir` → `hydra-runtime` → CLI. Its checks distinguish ordinary `Unit`/`Never` typing from D001 `Return`/`Break`/`Continue` and divergence effects. The AST type expression is `TypeExpr { name, span }`; its parser accepts only an identifier. The semantic type variants are `Int`, `Float`, `Bool`, `String`, `Unit`, `Never`. No aggregate or parameterized value/type exists. `FunctionId` and `SymbolId` are program-local; the current interpreter has a 128-call-depth cap and a 1,000,000-step budget.

Evidence anchors: `crates/hydra-lexer/src/lib.rs` recognizes `Dot` and greedily lexes `digits.digits` as Float; `crates/hydra-parser/src/lib.rs` owns expression Pratt calls, `()`/grouping, newlines, assignment target validation, 128 syntax-depth / 256 expression-depth gates; `crates/hydra-ast/src/lib.rs` owns type annotations; `crates/hydra-types/src/lib.rs` compares primitive structural identities and joins with `Never`; `crates/hydra-check/src/lib.rs` tracks path-sensitive D001 outcomes and typed expressions; `crates/hydra-resolve/src/lib.rs` owns lexical symbols; `crates/hydra-hir/src/lib.rs` encodes checked operations with spans; `crates/hydra-runtime/src/lib.rs` uses cloneable scalar `Value`, exact Rust float equality, and checked runtime fuel; `hydra-stdlib` exposes only primitive-printing `print` / `println`.

## Independent candidate decision matrix

Qualitative rankings describe current semantic burden; they are **not** numerical scores. Pipeline is the number/extent of compiler phases that must change, not difficulty measured in commits. **Leverage** measures reuse by future families. A candidate can be meaningful without another family even when its risk is high.

| Candidate | Hard dependencies and unresolved semantics | Semantic complexity | Pipeline | Runtime | Future leverage | Premature abstraction / lock-in | Independently bounded D002? |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Type-constructor foundation | Type application grammar, type argument arity, constructor identity/name lookup and builtin/user policy; **no accepted parameterized consumer** | Medium | High | Low | High | **High**: speculative grammar and namespace policy | **No**: cannot test a useful source program consuming one |
| **Tuple (selected)** | Parentheses/comma and Unit/grouping rules; ordered structural type; projection/equality/storage limits | Medium | High | Medium | High | Medium: product semantics, controlled dot syntax | **Yes**: construct, annotate, pass, return, project, compare |
| `List<T>` | Type application plus homogeneous element typing, literal/operations, alias/mutability, index and growth API | High | High | High | High | High: ownership/API decisions | No, under current single-family budget |
| `Set<T>` | Type application; block-vs-set braces; duplicate, membership, equality and deterministic iteration policies | Very High | High | High | Medium | Very High: key/iteration contract | No |
| Fixed `Array<T,N>` | First decide whether present; type application, fixed length `N` identity/validation, value and indexing contracts | Very High | High | High | Medium | Very High: const syntax/layout promise | No |
| Modules/imports | Compilation units, file resolution, name/visibility and cycles, cross-unit symbol identity | High | Very High | Medium | Very High | High: file/package semantics | In principle yes, but exceeds present scope |
| Algebraic data types | Nominal declarations, constructors and type identity, field/variant access and equality | Very High | Very High | High | Very High | High: declaration/namespace/representation choices | No for bounded D002 |
| Generics | Type/function binders, substitution, inference/instantiation, identity, runtime/backend policy | Very High | Very High | Medium–High | Very High | Very High: no concrete polymorphic consumer | No |

`List<T>` and `Set<T>` require a **type-argument contract** but do not require full user-defined generics. A hash protocol is **not** inherently required for sets: deterministic linear membership is possible, at a cost. `Array<T,N>` requires a constant-length contract but need not require general const generics. Generic functions could be designed without ADTs; ADTs could be defined without patterns/modules; modules could be independently valuable. None is an intrinsic hard prerequisite of Tuple. These distinctions are deliberate.

## Required vs convenient dependency graph

`REQUIRED` means a semantic decision needed by the feature as named, even if it can be designed inside that family. `WOULD BE NICE` denotes convenient reuse that must not be pulled in silently.

```text
POST-D001: lexer/parser/AST -> program-local resolver -> types/checker
           -> typed HIR -> interpreter with D001 flow outcomes

Type constructors
  REQUIRED: TypeExpr application, argument arity, constructor identity/resolution,
            builtin-vs-user constructor policy; standalone observable consumer unresolved
  WOULD BE NICE: accepted List/Set/Array consumer, user-defined generics

Tuple [SELECTED]
  REQUIRED: tuple literal/type grammar; Unit/grouping/singleton disambiguation;
            ordered structural product identity; read-only constant projection;
            immutable shared values, equality, bounded resource behavior
  WOULD BE NICE: destructuring, patterns, generic type applications, named fields

List<T>
  REQUIRED: type application contract; homogeneous elements; construction;
            storage/aliasing/mutability; indexing/bounds and core observable API
  WOULD BE NICE: user generics, iterators, dynamic collections framework

Set<T>
  REQUIRED: type application; non-block construction syntax; membership;
            deduplication by a declared equality rule; deterministic behavior
  WOULD BE NICE: hashing protocol, iterators, user-defined equality

Array<T,N>
  REQUIRED: inclusion decision; element type, constant N syntax/value;
            N in type identity; construction, bounds and ownership policy
  WOULD BE NICE: general const generics, native fixed layout optimization

Modules/imports
  REQUIRED: source/compilation-unit loader; module/visibility/namespace graph;
            cross-file stable identities, cycle/initialization policy
  WOULD BE NICE: package manager, incremental compiler, build cache

Algebraic data types
  REQUIRED: declaration and constructor forms; nominal identity/visibility;
            field/variant access and runtime representation/equality
  WOULD BE NICE: pattern matching, exhaustiveness, generic ADTs, modules

Generics
  REQUIRED: type binders and instantiation syntax; substitution and checking;
            inference boundaries and parametric identity; lowering policy
  WOULD BE NICE: ADTs, traits, monomorphization or other optimized backend
```

## Per-stage compiler pressure

`L`=lexer; `P`=parser; `A`=AST; `R`=resolver; `T`=types/checker; `H`=typed HIR; `X`=runtime; `D`=diagnostics; `C`=compatibility; `Q`=test/fuzz burden. `●` significant redesign/extension, `◐` localized or traversal work, `—` no expected direct change. Every row still needs full validation; these are review hypotheses, not implemented changes.

| Candidate | L | P | A | R | T | H | X | D | C | Q |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | --- | --- |
| Type constructors | ◐ | ● | ● | ● | ● | ◐ | — | ● | new type spellings | High; no executable consumer |
| **Tuple** | ◐ | ● | ● | ◐ | ● | ● | ● | ● | dot/float/grouping audit | High; depth, DAG, `NaN` |
| `List<T>` | ◐ | ● | ● | ◐ | ● | ● | ● | ● | delimiters and mutation | Very High; alias/bounds |
| `Set<T>` | ● | ● | ● | ◐ | ● | ● | ● | ● | `{}` versus blocks | Very High; determinism |
| `Array<T,N>` | ◐ | ● | ● | ◐ | ● | ● | ● | ● | brackets/length | Very High; length, sizes |
| Modules | ● | ● | ● | ● | ● | ● | ● | ● | unit/entry semantics | Very High; file graph |
| ADTs | ● | ● | ● | ● | ● | ● | ● | ● | declaration syntax | Very High; identity |
| Generics | ● | ● | ● | ● | ● | ● | ◐ | ● | syntax/binders | Very High; substitution |

The most important distinction is that Tuple has its own **source-visible consumer** (`t.0`). It does require changes to every major runtime/compiler layer, but each change is attributable to the single product-value family. Type constructors alone would prematurely force how future `List<T>`/`Set<T>` and user-defined generic constructors are parsed and resolved without any accepted value that exercises the result. Tuple literals alone without access would be inadequately useful; adding static projection is a narrow part of the same family, not patterns or generic indexing.

## Selected D002 contract (proposal only)

The full proposed decision, including grammar, diagnostics and tests, is [`002-tuples.md`](docs/decisions/002-tuples.md). Salient boundaries:

- `()` is existing Unit; `(x)` groups; `(x,)` is a one-element tuple; `(x, y[,])` has two elements; parenthesized type lists mirror values. Tuple arity and order determine structural identity, with no implicit conversions. `Never` remains normal-value bottom and D001's distinct effects remain intact.
- `t.0` and chained `t.0.1` are **constant read-only projections**; no destructuring, named fields, dynamic indexing, iteration, slicing, or mutation. Parser must disambiguate adjacent integer projections from existing floats (`1.25`) and keep statement/newline boundaries, call precedence, and no semicolons.
- Evaluate elements left-to-right exactly once, propagate D001 effects and stop on a non-normal path; locals copied into tuples are snapshot values, held as shared immutable aggregate storage. `print/println` still reject whole tuples with E3007; projected primitive fields remain printable.
- Equality is ordered/structural only for matching types. No ordering or hashing is introduced. Float equality retains IEEE semantics, including nested `NaN != NaN`; pointer-equal aggregates cannot be unconditionally equal.
- Max arity 64; max tuple type nesting 64; existing parser recursion 128, expression depth 256, call depth 128 and fuel 1,000,000 continue to apply. A runtime iterative worklist, shared pair memoization and fuel for each comparison visit prevent exponential walk of shared DAGs; opaque tuple host display prevents recursive output explosion. New **proposed** errors E3012 (projection of normal non-tuple) and E3013 (out-of-bounds index); existing E1101/E1102/E1104/E1105/E1106/E3002/E3003/E3004/E3007/E4006/E9003/E9004 keep their respective ownership.

## Adversarial selection challenge and safety resolution

| Challenge / possible hidden second family | Resolution and implementation proof obligation |
| --- | --- |
| `(x)` grouping, `()` Unit, `(x,)` singleton, `fn f(x: (Int,))` | First comma distinguishes tuple; source and type annotations follow same rule; no synthetic zero-element tuple. Test all forms and nested groups. |
| `t.0.1` scanned as Float `0.1`, or `1.25` scanned as projection | Lexer must recognize integer after projection-dot context and resume dot scanning, retaining all ordinary float tokens; regression tests for whitespace, chained calls and line boundaries. |
| `t.0` could imply lvalue, generic indexing or method dispatch | Read-only numeric projection only; `t.0 = v` uses E1104, `t.name` malformed; no other dot surface admitted. |
| Lack of patterns makes tuple dead data | `let t=(1,"x")`, typed function argument/return, `t.0`, and equality provide independent observable use. |
| `Never` erases D001 Break/Continue/Return path effects | Preserve checker structured outcome composition and left-to-right runtime flow for tuple elements and projections; test skipped side effects and nested loops. |
| Tuple values alias locals / mutate through references | Immutable snapshot of current value, shared aggregate clones; mutation of the source variable does not change tuple members, and there are no mutable references. |
| Shared pointer equality incorrectly makes nested NaN reflexive | Never shortcut an unexamined tuple pair solely on `Rc::ptr_eq`; traverse scalars, propagate IEEE float comparison and use `!=` as `!equals`. Check repeated shared subgraphs. |
| Logical exponential value expansion from shared subtrees | Store tuples with sharing; compare with iterative pair memoization and charge E4006 fuel for actual examined work. Compare DAGs with repeated shared edges and unmatched terminal fields. |
| Deep type trees / malformed HIR panic or exhaust host stack | Explicit 64-level tuple type, 64-field value bound; enforce 128/256 parse-expression limits; iterative/bounded helpers and E9004 checked invariant guards. |
| Recursive formatting or derived Rust traits perform hidden expansion | No source whole-tuple printing; host display opaque and bounded, explicit controlled equality implementation, review `Debug`/drop/copy paths. |
| Enormous arity, allocation, lexeme or index | Parser arity/depth and E1102 checked numeric bound, allocated bytes charged/checked; bounded fuzz/depth tests and real memory behavior review. |
| Compatibility with valid D001 source | Only formerly invalid comma-parentheses/positional projection become accepted; no reserved keywords; preserve old floats, grouping, Unit, operators, newline statements and CLI behavior. |

**Residual implementation risks (accepted for the design proposal, not waived):** need to implement/test the context-sensitive dot-versus-float scanner, bound type/render/walker and drop paths, confirm no `PartialEq` derived recursion escapes the controlled equality path, measure memory behavior on large shared/unique tuples, demonstrate E4006 for adversarial comparisons and show malformed HIR yields diagnostics rather than panics. If any obligation would force generic indexing, patterns, or a second semantic family, halt implementation for another human decision.

## Compatibility, required future work, and gates

Classification: **compatible extension** relative to the actual post-D001 source language. The grammar/typing of valid D001 programs and the frozen D001 behavior are unchanged in this documentation campaign. Existing source programs accepted before D002 must still evaluate identically when D002 is eventually implemented; a lexical collision that disproves this requires a human re-review. No D002 syntax is currently accepted.

Only after explicit **human acceptance** may a separate implementation campaign amend `spec/GRAMMAR.md`, `spec/TYPE_SYSTEM.md`, `spec/EXECUTION_MODEL.md`, `docs/ERROR_CODES.md` and (if needed for dot-number tokenization) `spec/LEXICAL_GRAMMAR.md`. It must implement Lexer → Parser/AST → Resolver → Types/Checker → HIR → Runtime in that order of semantic dependence, produce the positive/negative/UTF-8/resource/malformed-HIR cases in the proposal, run meaningful property/fuzz coverage and validate macOS/Linux/Windows, stable toolchain and MSRV 1.85.0. These are obligations for a **future authorized implementation**, not work performed in this gate.

This gate changes governance documentation only. D001 is **ACCEPTED / IMPLEMENTED / AUDITED / FROZEN / UNCHANGED**. Hydra 0.1 remains preserved; Hydra 0.2 is in progress; non-Tuple candidates, including H15 Lists/Sets/Arrays, remain deferred. **D002 IMPLEMENTATION NOT STARTED. D002 HUMAN APPROVAL REQUIRED.** No D003, tag, release, or production/specification change is authorized by this record.
