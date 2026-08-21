# Solingua

A compiled programming language that prioritizes explicitness and readability. Solingua compiles to native aarch64 assembly on macOS.

## Setup

### Prerequisites

- macOS on Apple Silicon (arm64)
- Xcode Command Line Tools (for the assembler and linker):
  ```bash
  xcode-select --install
  ```
- Rust toolchain:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Install

```bash
cargo install --path ./compiler
```

This installs the `solingua` binary to `~/.cargo/bin/`.

After making changes to the compiler, re-run `cargo install --path ./compiler` to update.

## Usage

```bash
solingua <source.sol>
```

This produces a native binary with the same name as the source file (minus the `.sol` extension).

### Verbose mode

```bash
solingua --verbose <source.sol>
```

Prints the full compilation pipeline: lexer output, abstract syntax tree, and generated assembly.

## Language Guide

### Functions

```
main -> {
    parameters()
    printLine("Hello world!")
}
    is function, returns nothing, project scope,
```

Every function is annotated with `is function, returns T, SCOPE scope,`. Functions with parameters:

```
max -> {
    parameters(
        a
            is value, type Integer, local scope,
        b
            is value, type Integer, local scope,
    )
    return if {
        a > b -> a
        else -> b
    }
}
    is function, returns Integer, project scope,
```

### Values (immutable)

```
greeting -> "Hello world!"
    is value, type String, local scope,
```

### Variables (mutable)

```
counter -> 0
    is variable, type Integer, local scope,

counter ~~> counter + 1
```

Variables are declared with `is variable` and mutated with the `~~>` operator ("now refers to").

### Scope labels

Every declaration carries a scope label:

| Scope | Meaning |
|-------|---------|
| `project scope` | Top-level, visible across the project |
| `local scope` | Inside a function body |
| `instance scope` | Inside a blueprint |

### If expressions (multi-branch)

Used to produce a value. Must include an `else` branch:

```
return if {
    a > b -> a
    else -> b
}
```

Branches can be nested:

```
return if {
    a < b -> if {
        b < c -> b
        else -> a
    }
    else -> b
}
```

### If statements (single-branch)

Used for conditional side effects. No `else` needed:

```
if x > 5 -> printLine(x)
```

### While loops

```
while counter < 10 {
    printLine(counter)
    counter ~~> counter + 1
}
```

### Comparison operators

`>`, `>=`, `<`, `<=`, `=`

### Arithmetic operators

`+`, `-`, `*`, `/` with standard precedence. Parentheses for grouping: `(2 + 3) * 4`

### Boolean operators

`and`, `or`, `not` — with short-circuit evaluation.

```
if x > 5 and x < 10 -> printLine(x)
```

### Blueprints (classes)

```
Cat -> {
    parameters(
        name,
            is value, type String, instance scope,
    )
    speak -> {
        parameters()
        printLine("Meow")
    }
        is function, returns nothing, instance scope,
}
    is blueprint,
```

Instantiation and usage:

```
cat -> Cat("Whiskers")
    is value, type Cat, local scope,
cat.speak()
printLine(cat.name)
```

### Interfaces (declared implementations)

```
Animal -> {
    name
        is value, type String, instance scope
    speak -> {
        parameters()
    }
        is function, returns nothing, instance scope
}
    is blueprint, declared implementation,

Cat -> {
    parameters(
        name,
            is value, type String, instance scope, full implementation,
            contracted,
    )
    speak -> {
        parameters()
        printLine("Meow")
    }
        is function, returns nothing, instance scope, full implementation,
        contracted,
}
    is blueprint, full implementation, type Animal,
```

### Singletons

```
Red
    is singleton
Green
    is singleton
Blue
    is singleton
```

Singletons are unique identity values with no fields or methods. Compare with `=`.

### Built-in functions

| Function | Description |
|----------|-------------|
| `printLine(value)` | Prints a value followed by a newline. Accepts String, Integer, and Boolean. |

## Examples

Run any example:

```bash
solingua examples/hello.sol
./examples/hello
```

Available examples:

- `hello.sol` — Hello world
- `variables.sol` — Values, variables, and mutation
- `arithmetic.sol` — Arithmetic operators and precedence
- `booleans.sol` — Boolean type and logical operators
- `max.sol` — Function parameters, if expressions
- `comparisons.sol` — Nested if expressions, multiple comparison operators
- `factorial.sol` — Recursion
- `if_statement.sol` — Single-branch if statements
- `loops.sol` — While loops with mutation
- `blueprint.sol` — Blueprints (classes) with methods and fields
- `interfaces.sol` — Declared and full implementation blueprints
- `singletons.sol` — Singleton values
