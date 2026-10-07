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

