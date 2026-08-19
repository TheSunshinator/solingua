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
    printLine("Hello world!")
}
    is function, returns nothing
```

Every function is annotated with `is function, returns T` (or `returns nothing`).

### Values

Values are immutable. They are declared with `is value, type T`:

```
greeting -> "Hello world!"
    is value, type String
```

### Parameters

Function parameters are declared inside a `parameters(...)` block:

```
max -> {
    parameters(
        a,
            is value, type Integer,
        b,
            is value, type Integer,
    )
    return if {
        a > b -> a
        else -> b
    }
}
    is function, returns Integer
```

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
if x > 5 -> printInteger(x)
```

### Comparison operators

`>`, `>=`, `<`, `<=`, `=`

### Built-in functions

| Function | Description |
|----------|-------------|
| `printLine(value)` | Prints a string followed by a newline |
| `printInteger(value)` | Prints an integer followed by a newline |

## Examples

Run any example:

```bash
solingua examples/hello.sol
./examples/hello
```

Available examples:

- `hello.sol` — Hello world
- `variables.sol` — Value declarations
- `max.sol` — Function parameters, if expressions, comparisons
- `comparisons.sol` — Nested if expressions, multiple comparison operators
- `if_statement.sol` — Single-branch if statements
