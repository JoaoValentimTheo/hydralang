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

## Hydra 0.2 accepted loop-control extension — implementation pending

The grammar above is the **Hydra 0.1 implemented grammar**. D001 accepts the following **Hydra 0.2 normative extension**, which has not yet been implemented in the lexer or parser:

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

**Compatibility: intentional breaking change.** Hydra 0.1 accepts `break` and `continue` as ordinary identifier spellings (e.g. `let break = 1` or `let continue = 2`). These previously valid identifier uses become invalid under the accepted Hydra 0.2 grammar. D001 specifies no other intentional invalidation of a valid Hydra 0.1 program. See `docs/decisions/001-loop-control.md`; implementation and compatibility regressions remain pending.
