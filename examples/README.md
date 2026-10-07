# Hydra 0.1 Examples

Every `.hyd` file in this directory is an executable Hydra 0.1 language-contract example. CI compiles and executes every example and asserts the exact output listed below.

| Example | Capability | Expected output |
| --- | --- | --- |
| `hello.hyd` | immutable binding, string, `println` | `Hydra` |
| `variables.hyd` | inference and explicit primitive annotation | `42`, `Hydra` |
| `mutability.hyd` | `let mut` and reassignment | `2` |
| `arithmetic.hyd` | precedence, grouping, left associativity | `7`, `9`, `5`, `2` |
| `booleans.hyd` | `!`, `&&`, `||`, short circuit | `true`, `false`, `true` |
| `conditionals.hyd` | `if`/`else` expression | `positive` |
| `blocks.hyd` | nested value-producing block | `42` |
| `functions.hyd` | typed parameters, return type, call | `42` |
| `recursion.hyd` | recursive calls | `120` |
| `while_loop.hyd` | loop, mutation, comparison | `0`, `1`, `2` |
| `strings.hyd` | string concatenation and Unicode | `Hydra λ` |
| `type_inference.hyd` | primitive local inference and Unit | `42`, `3.5`, `true`, `hydra`, `()` |
| `early_return.hyd` | early return and unreachable tail | `7` |
| `fibonacci.hyd` | recursion, arithmetic, comparison, loop | `0`, `1`, `1`, `2`, `3`, `5`, `8`, `13` |

The comma-separated values in the table denote separate output lines. None of these examples uses ordinary semicolon statement separators because semicolons are outside the Hydra 0.1 grammar.
