# Solingua Development Session

## What was built

### Compiler (Rust, targeting aarch64 macOS)
Built a full compiler from scratch that lexes, parses, type-checks, and generates native ARM64 assembly.

### Language features implemented (in order)
1. **Hello world** — `printLine("Hello world!")`
2. **Immutable values** — `is value, type String`
3. **Function parameters** — `parameters(...)` block
4. **If expressions** — multi-branch `if { cond then result, else then result }` and single-branch `if cond then expr`
5. **Comparison operators** — `>`, `>=`, `<`, `<=`, `=`
6. **User-defined functions** — with return types, recursive calls
7. **`printLine` overloading** — handles String, Integer, and Boolean
8. **`print` (no newline)** — for inline output
9. **Arithmetic operators** — `+`, `-`, `*`, `/` with correct precedence, parentheses
10. **Boolean operators** — `and`, `or`, `not` with short-circuit evaluation. `and`/`or` same precedence.
11. **Boolean type** — `true`/`false` literals, prints as "true"/"false"
12. **Blueprints (classes)** — `is blueprint`, constructor via malloc, member access, method calls
13. **Interfaces** — `declared implementation` vs `full implementation`, `contracted` members
14. **Singletons** — `Name is singleton`, unique identity values
15. **Generics** — `generics T`, `type Some of String`, generic field resolution
16. **For loops** — `for i from X to Y`, `include last`/`exclude last` (later removed)
17. **While loops** — `while condition { body }`
18. **Mutable variables** — `is variable` + `~~>` mutation operator (later changed to `becomes`)
19. **String templates** — `"Hello \(name)!"` with `\(expr)` interpolation
20. **Lists** — `List("a", "b", "c")`, `.at(index)` (1-based), `.size`
21. **Containers** — data-only structures with computed values
22. **Type checker** — runs between parser and codegen, catches type mismatches, undefined vars, wrong arg counts, mutability violations
23. **Error messages** — line/column numbers on parser errors, named variable errors from type checker

### Syntax evolution
The language syntax went through several major iterations:

**Phase 1: Arrow-based**
```
main -> { printLine("Hello world!") } is function, returns nothing,
greeting -> "Hello world!" is value, type String, local scope,
counter ~~> counter + 1
```

**Phase 2: Label-based (comma-separated)**
```
main is function, returns none, project scope, full implementation, specific, no contract,
    parameters(), means { printLine("Hello world!") }
```

**Phase 3: Block-based (current)**
```
let function main {
    labels[return(), generics[], mutable(false), visibility(public),
        scope(project), implementation(full), ]
    parameters{}
    body { printLine("Hello world!"); }
}
```

### Compiler architecture

**Lexer** (`src/lexer/`)
- `lexer.rs` — regex-based tokenizer, splits input at whitespace/symbol boundaries
- `keyword.rs` — keyword enum + `from(peek_fn)` pattern
- `symbol.rs` — symbols: `(){}[],+-*/;.`
- `literal.rs` — integers, strings (plain + templates), booleans
- `comparison.rs` — `>`, `>=`, `<`, `<=`, `=` as structured ComparisonOperator

**Parser** (`src/parser/`)
- `parser.rs` — recursive descent parser for declarations, statements, expressions
- `label.rs` — `parse_label_declaration()` returns `Vec<LabelDefinition>` with `Trivalent<T>` per label

**Label order**: return, generics, mutable, visibility, scope, implementation, contract

**AST** (`src/ast.rs`)
- `Program` → `Vec<Declaration>`
- `Declaration`: Function, Blueprint, Container, Singleton, Value
- `Statement`: ExpressionStatement, ValueDeclaration, MutationStatement, ReturnStatement, IfStatement, WhileLoop
- `Expression`: FunctionCall, StringLiteral, StringTemplate, IntegerLiteral, BooleanLiteral, ValueReference, MemberAccess, MethodCall, IfExpression, Comparison, Arithmetic, LogicalBinary, LogicalNot

**Codegen** (`src/codegen.rs`)
- Targets aarch64 macOS (Apple Silicon)
- Stack-based local variables with outgoing arg area at `[sp, #0..#15]`
- Variadic printf args go on the stack (Apple ARM64 ABI)
- Blueprint instances heap-allocated via `_malloc`
- Methods prefixed with blueprint name: `_Cat_speak`

**Type checker** (`src/typechecker.rs`)
- Validates types, mutability, argument counts, undefined variables
- Reports all errors at once (doesn't stop at first)

**Utility** (`src/util.rs`)
- `Trivalent<T>` — three-state enum: `NotApplicable`, `None`, `Some(T)` (for label parsing)

## Design decisions

- **Explicitness is the philosophy** — every declaration carries labels describing what it is
- **`return` label** serves double duty: return type for functions, value type for values
- **`mutable` on types** means extensible (can be subtyped), not reassignable
- **Labels have a fixed order**: return, generics, mutable, visibility, scope, implementation, contract
- **`Trivalent`** distinguishes "not specified" from "explicitly empty" from "has a value"
- **Syntax sugar** will be handled by IDE plugin in the future, not in the language itself
- **`instance { }` block** in types acts as a constructor body — loose statements run at construction time, `let function` declarations define methods

## What's next (not yet implemented)

- `let type` declarations (the new unified type system replacing blueprint/container/interface)
- `instance { }` blocks with mixed statements and method declarations
- `initially(expr)` for value initialization
- `Decimal` type and float literals
- `&` compound types (`Entity & Moveable`)
- `implementation(inherited)` for methods that use parent's implementation
- `becomes` for mutation inside instance bodies
- Imports / multi-file compilation
- Standard library

## Files

```
solingua/
├── compiler/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── ast.rs
│       ├── codegen.rs
│       ├── typechecker.rs
│       ├── util.rs
│       ├── lexer/
│       │   ├── mod.rs
│       │   ├── lexer.rs
│       │   ├── keyword.rs
│       │   ├── symbol.rs
│       │   ├── literal.rs
│       │   └── comparison.rs
│       └── parser/
│           ├── mod.rs
│           ├── parser.rs
│           └── label.rs
├── examples/
│   ├── hello.sol
│   ├── factorial.sol
│   ├── lists.sol
│   ├── type.sol
│   └── ...
└── README.md
```
