# Hydra 0.1 Grammar

Status: **IMPLEMENTED for the core subset**.

```text
program      := newline* function* EOF
function     := "fn" IDENT "(" params? ")" ("->" type)? block
params       := param ("," param)*
param        := IDENT ":" type
type         := IDENT
block        := "{" newline* block-entry* "}"
block-entry  := let-stmt | while-stmt | return-stmt | expression
let-stmt     := "let" "mut"? IDENT (":" type)? "=" expression
while-stmt   := "while" expression block
return-stmt  := "return" expression?
expression   := Pratt expression over assignment, boolean, comparison,
                arithmetic, unary, call, if, block, literals, and names
```

Assignment is right-associative and only a resolved mutable local may be assigned. Calls bind tighter than unary and binary operators.

From loosest to tightest, expression precedence is:

1. assignment `=` (right-associative)
2. `||`
3. `&&`
4. equality `==`, `!=`
5. relational `<`, `<=`, `>`, `>=`
6. additive `+`, `-`
7. multiplicative `*`, `/`, `%`
8. unary `-`, `!`
9. call `(...)`

Binary operators are left-associative. Newlines delimit adjacent block entries, but the parser permits layout newlines inside parentheses and calls, before `else`, and after a binary operator while its right operand is still required. A semicolon is an unexpected character (E1001), not a statement separator. Two otherwise valid adjacent statements on one line without an allowed grammar boundary are rejected as syntax (E1101).

The parser enforces two independent structural guards before later recursive compiler phases consume the AST. Recursive syntax nesting is limited to 128 and reports E1105. Constructed expression-tree depth is limited to 256 and reports E1106; this second guard also covers long left-associative chains whose Pratt parsing itself does not become deeply recursive.

## Hydra 0.2 accepted loop-control extension — D001 implemented and frozen

The grammar above is the **historical Hydra 0.1 grammar**. D001 accepted the following **Hydra 0.2 normative extension**, now implemented and frozen; see `HYDRA_0_2_D001_FREEZE.md`:

```text
block-entry  := let-stmt
              | while-stmt
              | return-stmt
              | break-stmt
              | continue-stmt
              | expression
break-stmt   := "break"
continue-stmt := "continue"
```

`break` and `continue` become reserved keywords, not identifiers. Each is a value-less statement, never an expression, without operands, values, labels, named targets, or call forms. They follow the existing `return` statement's newline or closing-`}` boundary rule (including normal comment/newline handling). Semicolons remain forbidden. `break 1`, `continue 1`, `break(label)`, `continue(label)`, labels, and attempted call-like or expression use are invalid syntax (E1101); source placement legality is separately checked by the type checker (E3010/E3011).

**Compatibility: intentional breaking change.** Hydra 0.1 accepted `break` and `continue` as ordinary identifier spellings (e.g. `let break = 1` or `let continue = 2`). These previously valid identifier uses became invalid under D001. D001 specifies no other intentional invalidation of a valid Hydra 0.1 program. See `docs/decisions/001-loop-control.md` and the completed regression audit in `HYDRA_0_2_D001_FREEZE.md`.

## Hydra 0.2 D002 implemented structural tuples

**2026-10-08 approved Option A clarification:** The parser owns E1105 for
depth >64 in **written** tuple types. The checker owns E3014 for depth >64 in
**inferred** tuple types, with the constructing expression span and actual/
maximum depths. A primitive has tuple depth zero and grouping adds no layer.
The following grammar is implemented in D002; historical acceptance wording remains in the decision record.

The following is the **locked normative D002 grammar**, accepted on 2026-10-08. It extends the implemented post-D001 language; **tuple parsing, type syntax and projection are implemented**. Existing 0.1/D001 syntax and behavior remain in force alongside this extension.

```text
type             := IDENT | "(" ")" | "(" type ")" | tuple-type
tuple-type       := "(" type "," (type ("," type)* ","?)? ")"
tuple-expression := "(" expression "," (expression ("," expression)* ","?)? ")"
group-expression := "(" expression ")"
unit-expression  := "(" ")"
postfix          := primary (call-suffix | projection-suffix)*
projection-suffix := "." DECIMAL_INDEX
DECIMAL_INDEX    := ASCII_DIGIT+  // unsigned base-10, bounded integer index
```

An empty `()` is existing `Unit` in value and type positions; a single `(x)` or `(Int)` groups without creating a tuple. **A comma is required to create every nonempty tuple**: `(x,)` and `(Int,)` have one field, `(x,y)` and `(Int,String)` have two, and a trailing comma is allowed when arity is at least two. Tuple expressions and tuple types may nest, with structural nesting limits below. Semicolons are invalid; commas separate only elements inside parentheses. Parenthesized newlines follow the existing grouping/call layout policy.

Constant projections such as `t.0`, `t.1`, `(pair()).0` and `t.0.1` are left-associated postfix operations at **the same highest precedence as calls**, and may chain with calls where already valid. The dot and index must occur on the same logical line as the base; intervening spaces on that line are permitted (`t . 0`). Only nonnegative decimal integer indices are admitted. Leading zeroes are permitted and denote the same index (`.00` = `.0`). `t.-1`, `t.name`, `t.(i)`, `t[i]`, and `t.0 = value` are invalid: malformed suffixes report E1101, while a projection used as an assignment target reports E1104. Dynamic/generic indexing and field mutation are outside D002.

**Normative lexer disambiguation:** ordinary `1.25` remains one `Float` token. Following a projection `Dot`, a decimal digit run is a distinct `Int` token **without absorbing** the next `.digit` sequence. Thus `t.0.1` is `Identifier Dot Int Dot Int`, and `t.1.5` is two chained integer projections, never `Identifier Dot Float`. The implementation may use narrowly scoped after-dot lexical context; it must not alter general float scanning, reserve new keywords or reinterpret existing valid source.

At most **64 fields** may occur in a tuple expression or type; arity 65 reports E1101 with the tuple syntax span. Tuple **type nesting** is limited to **64 tuple layers along any path**, with excess reported as E1105. Existing `MAX_PARSE_DEPTH = 128` (E1105) and `MAX_EXPR_DEPTH = 256` (E1106) remain unchanged. All tuple children and projection bases participate in the iterative expression-depth check, and tuple-type parsing must obey the syntax/nesting guards and normal progress/recovery rules. Oversized projection integer indices report E1102 without truncation.

This is a **compatible extension** against frozen D001: no new keywords, no changed `()`/grouping semantics, no changed float literals or previously accepted calls. Destructuring, patterns, generic type application, lists, sets, arrays, named fields and other family syntax are not accepted. See `docs/decisions/002-tuples.md`; the implementation is documented in `HYDRA_0_2_D002_IMPLEMENTATION.md`.
