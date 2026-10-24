# KERNYX Syntax Design

Status: draft (0.1 frozen; post-0.1 constructs provisional)

This document records the syntax decisions for KERNYX and realistic syntax samples
validated against them. It is project-specific: no keyword is treated as a feature by
itself; each construct's grammar, typing rules, evaluation semantics, diagnostics, and
interactions are defined in docs/GRAMMAR.md, docs/SEMANTICS.md, and docs/SPEC.md.

## 1. Vocabulary (0.1, implemented)

| Keyword | Meaning | Implemented? |
|---|---|---|
| `app` | Optional application metadata at file top | yes |
| `fn` | Function declaration | yes |
| `let` | Immutable local binding | yes |
| `give` | Return a value from a function | yes |
| `when` / `otherwise` | Conditional branching | yes |
| `Text`, `Int`, `Float`, `Bool`, `Unit` | Primitive types | yes |
| `true`, `false` | Boolean literals | yes |

The following are lexed as keywords but rejected by the parser as proposed syntax
(not implemented in 0.1): `flow`, `data`, `expose`, `return`.

## 2. Design principles

- Familiar enough for an experienced programmer to read immediately.
- A small number of distinctive constructs (`give`, `when`/`otherwise`, `app`) give the
  language its own voice; most syntax follows familiar C-like conventions.
- No keyword is renamed for novelty. `return` is deliberately NOT used; `give` is the
  return construct, and its exact interaction with return-path analysis is specified in
  docs/SEMANTICS.md.
- Avoid both needless ceremony and magical implicit behavior (no implicit Int-to-Float
  coercion in 0.1).
- Syntax decisions must not make future features (generics, pattern matching, modules,
  effects, async) ambiguous or force broad parser rewrites.

## 3. Realistic syntax samples (0.1)

### 3.1 Hello world (verified, examples/hello.knx)

```
app "hello"

fn greet(name: Text) -> Text {
    give "yo, " + name
}

fn main() {
    let user = greet("world")

    when user != "" {
        print(user)
    } otherwise {
        print("nah, nothing here")
    }
}
```

### 3.2 Arithmetic with precedence (verified, examples/arithmetic.knx)

```
app "arithmetic"

fn main() -> Unit {
    print(2 + 3 * 4)
    print((2 + 3) * 4)
    print(10 - 2 - 3)
    print(7 / 2)
    print(-3 + 4)
    print(2 == 2)
    print(3 != 3)
    print(true && false)
    print(true || false)
}
```

### 3.3 Calls between user functions (verified, examples/functions.knx)

```
app "functions"

fn double(x: Int) -> Int {
    give x * 2
}

fn add(a: Int, b: Int) -> Int {
    give a + b
}

fn main() -> Unit {
    let result = double(add(3, 4))
    print(result)
}
```

### 3.4 Invalid program and diagnostic (verified, examples/errors/bad_type.knx)

```
fn main() -> Unit {
    let name: Text = 42
}
```

Diagnostic:

```
error[KX0203]: declared type `Text` does not match initializer type `Int`
 --> examples/errors/bad_type.knx:2:22
  |
2 |     let name: Text = 42
  |                        ^^
  |
  = help: remove the annotation or change the initializer
```

### 3.5 Post-0.1 samples (proposed, not implemented)

These illustrate intended direction; none compile in 0.1.

Data type and pattern matching (proposed `data`):

```
data Shape {
    Circle(radius: Float),
    Rect(width: Float, height: Float)
}

fn area(s: Shape) -> Float {
    when s {
        Circle(r) => 3.14 * r * r
        Rect(w, h) => w * h
    }
}
```

Structured concurrent work (proposed `flow`):

```
flow fetch_all(urls: List<Text>) -> List<Text> {
    // proposed semantics: structured, cancellable concurrent work
}
```

Explicit API boundary (proposed `expose`):

```
expose greet(name: Text) -> Text
```

## 4. Interaction notes

- `let` bindings are immutable and block-scoped; shadowing is allowed in inner scopes
  and bindings do not leak out of their block.
- `print` is a builtin taking exactly one argument of any printable type; it writes to
  stdout.
- `main` is the entry point and may omit `-> Unit` (defaults to `Unit`).
- Non-`main` functions require an explicit return type; `give` is only valid in
  non-`Unit` functions.
