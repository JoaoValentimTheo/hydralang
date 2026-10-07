# Lexical Grammar

Status: **IMPLEMENTED for the 0.1 core subset**.

The lexer recognizes identifiers, keywords, integer and floating-point literals, string literals, punctuation, operators, `//` line comments, newlines, and EOF. Newlines are tokens used as statement boundaries; they are ignored where the parser explicitly permits layout.

Identifiers begin with `_` or an alphabetic Unicode scalar value and continue with `_` or alphanumeric Unicode scalar values. Unexpected characters and unterminated strings produce E1xxx diagnostics. The lexer must always advance on valid UTF-8 input.

