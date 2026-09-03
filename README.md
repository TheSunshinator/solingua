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

## Definitions
Here is a table with some definition of the meaning of some words in the context of Solingua

| Word | Definition |
| :---: | :--- |
| Construct | Anything that can be declared in the language. A value, a function, a type, etc |
| Folder | Refering to folders in the file system |
| package | All files that are going to be compiled together in a single binary |


## Language Guide
### Identifiers
Grammar:
```
digit                  = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
letter                 = "a"..."z" | "A"..."Z" ;
identifier             = letter , { letter | digit | "_" } ;
typeIdentifier         = identifier , ("<" , identifier , ("," , identifier)* , ">") ;
genericTypeIdentifiers = identifier , ("return[" , typeIdentifier , ("&" , typeIdentifier)* , "]" ;
```

### Labels
Labels are places after a declaration, separated by commas. They should appear in the following order.
Some are optional depending on the context
- return
- generic
- mutable
- visibility
- scope
- implementation
- contract

Grammar: 
```
labels = "labels[" , (identifier , labelParameters)*, "]" ;
labelParameters = labelParameterList | labelStandardParameter ;
labelStandardParameter = "(" , identifier , ")" ;
labelParameterList = "[" , identifier , "]" ;
```

#### return
Required label used to associate a (return) type to a construct. For values, it _must_ include a type; It cannot be empty.

| Construct |            Meaning            |
|:---------:|:-----------------------------:|
|   value   |       Type of the value       |
| function  | Type returned by the function |
|   type    |   Parent type and contracts   |

Grammar: `returnLabel = "return(" , (typeIdentifier), "), " ;`

Examples: 
```
labels[return(), …]
labels[return(Integer), …]
```

#### generic

Label required for types and functions constructs declaring generic types that they use.

Grammar:
```
genericLabel = "generic(" , (genericTypeIdentifiers , ("," , genericTypeIdentifiers)*) , "), " ;
```

Example: 
```
labels[…, generic[], …]
labels[…, generic[T], …]
labels[…, generic[T, U], …]
labels[…, generic[T return ViewModel, U], …]
labels[…, generic[T return ViewModel & Listener, U], …]
```

#### mutable
Required label that denotes a construct that can be mutated or not. On a function, it means that it can or cannot be overridden. 

| Construct |          Meaning           |
|:---------:|:--------------------------:|
|   value   |  Value can be reassigned   |
| function  | Function can be overridden |
|   type    |    Type can be extended    |

Grammar: `mutabilityLabel = "mutable(" , ("true" | "false") , "), " ;`

Example:
```
let value counter {
  labels[return(String), mutable(true), …]
  initially(0)
}

counter becomes counter + 1

let value greeting {
  labels[return(String), mutable(false), …]
  initially("Hello world!")
}

```

#### visibility
Required for any construct defined directly in a file, and instance scoped declarations,
it defined what other constructs can access this one.

| Visibility | Meaning                                                                                                                                                                                    |
|:----------:|:-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|  private   | For constructs declared directly in files, it can only be accessed by other construct in that file.<br/>For constructs in a type construct, it can only be accessed from within that type. |
|   public   | No restriction                                                                                                                                                                             |
|   folder   | Can be accessed from any construct defined in the same folder                                                                                   Type can be extended                       |
| subfolders | Can be accessed from any construct defined in the same folder or subfolders                                                                                                                |
|  package   | Can only be accessed from within this package                                                                                                                                              |
|  contract  | Can only be accessed from child types                                                                                                                                                      |

Grammar: `visibilityLabel = "visibility(" , { "private" | "public" | "folder" | "subfolders" | "package" | "contract" } , "), " ;`

Example: ```labels[…, visibility(public), …]```

#### scope
Required for all declarations, used to determine the scope of the declaration.

|  Scope   | Meaning                                       |
|:--------:|-----------------------------------------------|
| project  | Top-level, not tied to anything               |
|  local   | Only exist in the immediate surrounding block |
| instance | Tied to an instance of a type                 |
|   type   | Tied to a type identifier                     |

Grammar: `scopeLabel = "scope(" , { "instance" | "local" | "project" | "type" } , "), " ;`

Example: ```labels[…, scope(local), …]```

#### implementation
Required for constructs with a `scope(instance)`. Used to determine at what degree the construct is implemented

| Implementation | Meaning                                                                                              |
|:--------------:|------------------------------------------------------------------------------------------------------|
|      full      | The construct is fully implemented                                                                   |
|    partial     | The construct is partially implemented, either unfinished or mixes full/no implementation constructs |
|      none      | The construct is declared, but provides no implementation                                            |
|   inherited    | The construct is declared, but provides no implementation. The ancestor does and is unchanged.       |

Grammar: `implementationLabel = "implementation(" , { "full" | "partial" | "none" } , "), " ;`

Example: ```labels[…, implementation(partial), …]```

#### contract
Required for constructs with a `scope(instance)`. Used to determine if a construct is required by contract, or it's a new construct

Grammar: `contractLabel = "contract(" , typeIdentifier , "), " ;`

Example: ```labels[…, contract(Listener), …]```

### Parameters
Defined parameters for a function or a type constructor

Grammar:
```
parameterList = "parameters {" , parameter* , "}" ;
parameter = valueDefinition | functionDefinition
```

### Functions

Grammar:
```
"let function " , identifier , "{" ,
  labels ,
  parameterList ,
  "body {" , statement* , "}"
"}" ;
```

Examples:
```
let function main {
    labels[return(), generics[], mutable(false), visibility(public),
        scope(project), implementation(full), ]
    parameters{}
    body { printLine("Hello world!"); }
}

let function main {
    labels[return(Integer), generics[], mutable(false), visibility(public),
        scope(project), implementation(full), ]
    parameters {
        let value a {
            labels[return(Integer), mutable(false), scope(local), implementation(full), ]
        }
        let value b {
            labels[return(Integer), mutable(false), scope(local), implementation(full), ]
        }
    }
    body { 
        return if {
            a > b then a;
            else then b;
        };
    }
}
```

### Values

Grammar:
```
"let value " , identifier , "{" ,
  labels ,
  ("initially(" , statement , ")"
"}"
```

Example:
```
let value greeting {
    labels[return(String), mutable(false), scope(local), implementation(full), ]
    initially("Hello world!")
}
let value counter {
    labels[return(Integer), mutable(true), scope(local), implementation(full), ]
    initially(0)
}

counter becomes counter + 1;
```

### If expressions (multi-branch)

Used to produce a value. If used to return or initiate a value, they must include an `else` branch:

```
if {
    someValue = "foo" then printLine("Foo!");
};
return if {
    a > b then a;
    else then b;
};
```

Branches can be nested:

```
return if {
    a < b then if {
        b < c then b;
        else then a;
    }
    else then b;
};
```

### If statements (single-branch)

Used for conditional side effects. No `else` needed:

```
if x > 5 then printLine(x);
```

### While loops

```
while counter < 10 {
    printLine(counter);
    counter becomes counter + 1;
}
```

### Comparison operators

`>`, `>=`, `<`, `<=`, `=`

### Arithmetic operators

`+`, `-`, `*`, `/` with standard precedence. Parentheses for grouping: `(2 + 3) * 4`

### Boolean operators

`and`, `or`, `not` — with short-circuit evaluation. 
`not` precedes `and` and `or`.
`and` and `or` have the same precedence.

```
if x > 5 and x < 10 then printLine(x)
```

### Type
#### Full implementation
```
let type Cat {
    labels[return(), generics[], mutable(false), visibility(public),
      scope(project), implementation(full), ]
    parameters {
        let value name {
            labels[return(String), mutable(false), scope(instance), implementation(full), ]
        }
    }
    instance {
        let function speak {
            labels[return(), generics[], mutable(false), visibility(public),
                scope(instance), implementation(full), contract()]
            parameters {}
            body { printLine("Meow"); }
        }
    }
}
```

Instantiation and usage:

```
let value cat {
    labels[return(Cat), generics[], mutable(false), visibility(public),
        scope(local), implementation(full),]
    initially(Cat("Krokmou"))
}
cat.speak()
printLine(cat.name)
```

#### No implementations

```
let type Animal {
    labels[return(), generics[], mutable(true), visibility(public),
        scope(project), implementation(none),]
    instance {
        let value name {
            labels[return(String), mutable(false), scope(instance), implementation(none), ]
        }
        let function speak {
            labels[return(), generics[], mutable(false), visibility(public),
                scope(instance), implementation(full)]
            parameters {}
        }
    }
}

let type Cat {
    labels[return(Animal), generics[], mutable(false), visibility(public),
      scope(project), implementation(full), ]
    parameters {
        let value name {
            labels[return(String), mutable(false), scope(instance), implementation(full), contract(Animal)]
        }
    }
    instance {
        let function speak {
            labels[return(), generics[], mutable(false), visibility(public),
                scope(instance), implementation(full), contract(Animal)]
            parameters {}
            body { printLine("Meow"); }
        }
    }
}
```

Instantiation and usage:

```
let value cat {
    labels[return(Animal), generics[], mutable(false), visibility(public),
        scope(local), implementation(full),]
    initially(Cat("Krokmou"))
}
cat.speak()
printLine(cat.name)
```

### Singletons

```
let singleton Red {
    labels[return(), visibility(public), scope(project)]
}
let singleton Red {
    labels[return(), visibility(public), scope(project)]
}
let singleton Red {
    labels[return(), visibility(public), scope(project)]
}
```

Singletons are unique identity values with no fields or methods. Compare with `=`.


### Built-in functions

| Function           | Description                                                                 |
|--------------------|-----------------------------------------------------------------------------|
| `printLine(value)` | Prints a value followed by a newline. Accepts String, Integer, and Boolean. |
| `readInput()`      | Wait for user input from the console                                        |

## Examples

Run any example:

```bash
solingua examples/hello.sol
./examples/hello
```

Available examples:

- `hello.sol` — Hello world
- `factorial.sol` — Recursion
