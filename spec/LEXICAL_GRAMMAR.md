# Lexical Grammar

Status: **IMPLEMENTED for the 0.1 core subset**.

The lexer recognizes identifiers, keywords, integer and floating-point literals, string literals, punctuation, operators, `//` line comments, newlines, and EOF. Newlines are real tokens used as statement boundaries; they are consumed as layout only where the parser explicitly permits it. Semicolon is not a Hydra 0.1 token and therefore reports E1001.

Identifiers begin with `_` or an alphabetic Unicode scalar value and continue with `_` or alphanumeric Unicode scalar values. Unexpected characters and unterminated strings produce E1xxx diagnostics. The lexer must always advance on valid UTF-8 input.

Integer tokens contain digits only; a leading `-` is lexed separately as unary negation. The parser then requires the positive literal token itself to fit in signed 64-bit range. This is why the direct spelling `-9223372036854775808` reports E1102 even though the resulting mathematical value would equal `i64::MIN`.
