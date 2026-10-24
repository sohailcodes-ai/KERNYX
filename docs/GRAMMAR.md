# KERNYX Grammar

Status: 0.1 frozen; post-0.1 productions marked provisional

Notation: EBNF-like. `*` = zero or more, `+` = one or more, `?` = optional,
`|` = alternation, `[...]` = character class, "..." = literal text.
Terminals are in quotes; non-terminals are italic.

## 0.1 grammar (implemented)

```
program       := app_decl? function*
app_decl      := "app" text_literal
function      := "fn" IDENT "(" params ")" ("->" type)? "{" block "}"
params        := param ("," param)*
param         := IDENT ":" type
block         := "{" statement* "}"
statement     := let_stmt | give_stmt | when_stmt | block_stmt | expr_stmt
let_stmt      := "let" IDENT (":" type)? "=" expression ";"
give_stmt     := "give" expression ";"
when_stmt     := "when" expression block ("otherwise" block)?
block_stmt    := "{" block "}"
expr_stmt     := expression ";"
expression    := binary_expr
binary_expr   := unary_expr (binop unary_expr)*
binop         := "||" | "&&" | "==" | "!=" | "<" | "<=" | ">" | ">="
                | "+" | "-" | "*" | "/" | "%"
unary_expr    := "-" unary_expr | "+" unary_expr | "!" unary_expr | primary
primary       := int_literal | float_literal | text_literal | "true" | "false"
                | IDENT | "(" expression ")" | IDENT "(" arguments ")"
arguments     := expression ("," expression)*
type          := "Int" | "Float" | "Bool" | "Text" | "Unit"
```

### Precedence (low to high)

1. `||`
2. `&&`
3. `==`, `!=`
4. `<`, `<=`, `>`, `>=`
5. `+`, `-`
6. `*`, `/`, `%`
7. unary `-`, `+`, `!`
8. primary (literals, identifiers, parenthesized expressions, calls)

All binary operators are left-associative.

## Provisional grammar (post-0.1, NOT implemented)

These productions are design material only. `flow`, `data`, `expose`, `return` are
rejected by the 0.1 parser.

```
data_decl     := "data" IDENT "{" variant* "}"
variant       := IDENT "(" params? ")"

flow_decl     := "flow" IDENT "(" params ")" "->" type "{" block "}"

expose_decl   := "expose" IDENT "(" params ")" "->" type ";"

return_stmt   := "return" expression ";"

match_arm     := pattern "=>" expression
pattern       := IDENT "(" pat_binding* ")" | literal | "_"
```

## 1. What is intentionally excluded from 0.1

- Generics, modules, imports/exports.
- Mutable bindings (`let mut`).
- Pattern matching beyond `when`/`otherwise`.
- Async/await, `flow`, structured concurrency.
- Effects/capability annotations.
- Native machine-code generation, bytecode, WASM.

## 2. Why this grammar is stable for 0.1

It is small (one expression level with precedence climbing), has no nested
constructs that would require backtracking, and leaves clear extension points:
- generics can be added to `type` and `function`;
- pattern matching can extend `when` arms;
- modules can extend `program`;
- effects can extend `function` and `call` without changing expression precedence.
