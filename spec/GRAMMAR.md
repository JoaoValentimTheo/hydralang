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

The parser enforces a maximum recursive syntax nesting depth of 128 and reports E1105 before pathological input can consume the host stack.
