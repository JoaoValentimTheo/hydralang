# Lexical Grammar

Status: **IMPLEMENTED for the 0.1 core subset**.

The lexer recognizes identifiers, keywords, integer and floating-point literals, string literals, punctuation, operators, `//` line comments, newlines, and EOF. Newlines are real tokens used as statement boundaries; they are consumed as layout only where the parser explicitly permits it. Semicolon is not a Hydra 0.1 token and therefore reports E1001.

Identifiers begin with `_` or an alphabetic Unicode scalar value and continue with `_` or alphanumeric Unicode scalar values. Unexpected characters and unterminated strings produce E1xxx diagnostics. The lexer must always advance on valid UTF-8 input.

Integer tokens contain digits only; a leading `-` is lexed separately as unary negation. The parser then requires the positive literal token itself to fit in signed 64-bit range. This is why the direct spelling `-9223372036854775808` reports E1102 even though the resulting mathematical value would equal `i64::MIN`.

## Hydra 0.2 D002 implemented dot-index lexical rule

Existing ordinary floating-point literals such as `1.25` remain a single `Float` token with unchanged numeric semantics. D002 permits constant positional projection with `Dot` followed by an unsigned ASCII decimal integer index. **After a projection dot**, the following digit run must tokenize as `Int` without consuming a subsequent `.digit` sequence. Consequently `t.0.1` and `t.1.5` tokenize as `Identifier Dot Int Dot Int`, whereas `1.25` is still `Float`. Leading zeros in projection indices are accepted. A narrow lexical context after `Dot` implements this rule; global number scanning, other valid float tokenizations, identifier rules, newline boundaries and the absence of semicolons remain unchanged. D002 is implemented and frozen.

## Hydra 0.2 D003 accepted List lexical contract — NOT IMPLEMENTED

The 2026-10-09 accepted D003 extension reuses existing `[` and `]` punctuation for List literals and readonly postfix indexing. `List` remains an ordinary identifier token; it is recognized as a builtin type constructor **only in type positions**, without reserving a new keyword or changing the meaning of `List` as an existing value identifier. The existing `<` and `>` operator tokens serve as contextual type argument delimiters within `List<T>`; adjacent `>>` is parsed as two closing `>` tokens in nested type positions, **not** a shift operator. Expression comparisons and all already-accepted float/projection tokenization remain unchanged.

Commas delimit List elements, with an optional trailing comma; semicolons remain invalid. Newline tokens keep their existing statement-boundary meaning. A leading `[` on a subsequent statement must never be absorbed as indexing of a preceding completed expression. This is an accepted specification for a future compiler change; it does **not** describe currently executable List syntax. See `HYDRA_0_2_D003_ACCEPTANCE.md`.
