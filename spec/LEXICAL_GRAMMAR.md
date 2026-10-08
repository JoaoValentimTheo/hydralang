# Lexical Grammar

Status: **IMPLEMENTED for the 0.1 core subset**.

The lexer recognizes identifiers, keywords, integer and floating-point literals, string literals, punctuation, operators, `//` line comments, newlines, and EOF. Newlines are real tokens used as statement boundaries; they are consumed as layout only where the parser explicitly permits it. Semicolon is not a Hydra 0.1 token and therefore reports E1001.

Identifiers begin with `_` or an alphabetic Unicode scalar value and continue with `_` or alphanumeric Unicode scalar values. Unexpected characters and unterminated strings produce E1xxx diagnostics. The lexer must always advance on valid UTF-8 input.

Integer tokens contain digits only; a leading `-` is lexed separately as unary negation. The parser then requires the positive literal token itself to fit in signed 64-bit range. This is why the direct spelling `-9223372036854775808` reports E1102 even though the resulting mathematical value would equal `i64::MIN`.

## Hydra 0.2 D002 accepted dot-index lexical rule — implementation pending

Existing ordinary floating-point literals such as `1.25` remain a single `Float` token with unchanged numeric semantics. D002 permits constant positional projection with `Dot` followed by an unsigned ASCII decimal integer index. **After a projection dot**, the following digit run must tokenize as `Int` without consuming a subsequent `.digit` sequence. Consequently `t.0.1` and `t.1.5` tokenize as `Identifier Dot Int Dot Int`, whereas `1.25` is still `Float`. Leading zeros in projection indices are accepted. A narrow lexical context after `Dot` may implement this rule; global number scanning, other valid float tokenizations, identifier rules, newline boundaries and the absence of semicolons remain unchanged. This is **accepted normative behavior only**, with lexer implementation pending separate authorization.
