<p align="center"><img width="300px" src="https://pen-lang.org/icon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](#license)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is the parallel, concurrent, and functional programming language focused on application programming following [Go][go]'s philosophy. It aims for further simplicity, testability, and portability to empower team (v. individual) and/or long-term (v. short-term) productivity.

Its syntax, type system, [effect system](#dynamic-effect-system), and module system are fashioned to achieve those goals being simple and easy to grasp for both newcomers and experts. One of the biggest differences from the other functional languages is [polymorphism without generics](#polymorphism-without-generics).

Pen provides [the two built-in functions of `go` and `race`][concurrency] to represent many concurrent/parallel computation patterns. Thanks to its syntax, type system, and [the state-of-the-art reference counting garbage collection][gc], programs are always data-race free.

System libraries and runtime in Pen are detachable from applications. Therefore, Pen can compile the same applications even for [WebAssembly](https://webassembly.org/) and [WASI](https://wasi.dev/). Pen also provides [Rust][rust]/C FFI to reuse existing resources written in those languages.

```pen
import Core'Number
import Os'File

# The `\` prefix for λ denotes a function.
findAnswer = \(kind string) number {
  # Secret source...

  21
}

main = \(ctx context) none {
  # The `go` function runs a given function in parallel.
  # `x` is a future for the computed value.
  x = go(\() number { findAnswer("humanity") })
  y = findAnswer("dolphins")

  _ = File'Write(ctx, File'StdOut(), Number'String(x() + y))

  none
}
```

## Install

Pen is available via [Homebrew](https://brew.sh/).

```sh
brew install pen-lang/pen/pen
```

For more information, see [Install][install].

## Examples

See [the `examples` directory](examples).

## Documentation

- [Getting started][install]
- Guides
  - [Building an executable](https://pen-lang.org/guides/building-an-executable)
  - [Creating a library](https://pen-lang.org/guides/creating-a-library)
  - [Using a library](https://pen-lang.org/guides/using-a-library)
- Language reference
  - [Syntax](https://pen-lang.org/references/language/syntax)
  - [Types](https://pen-lang.org/references/language/types)

## Comparison with [Go][go]

### Overview

|                   | Pen                      | Go                           |
| ----------------- | ------------------------ | ---------------------------- |
| Domain            | Application programming  | System programming           |
| Paradigm          | Functional               | Imperative / object-oriented |
| Memory management | [Reference counting][gc] | Concurrent mark-and-sweep    |
| System library    | Your choice!             | Built-in                     |
| Values            | Immutable                | Mutable                      |

### Runtime

|                        | Pen                                          | Go                                   |
| ---------------------- | -------------------------------------------- | ------------------------------------ |
| Context switch         | [Continuations](#context-switch)             | Platform dependent                   |
| Concurrent computation | [Built-in functions][concurrency]            | `go` expression                      |
| Synchronization        | Futures, lazy lists                          | Channels, concurrent data structures |
| Data race prevention   | Built into [GC][gc]                          | Dynamic analysis                     |
| Resource management    | Built into [GC][gc]                          | `defer` statement                    |
| Error handling         | `error` type, [`?` operator][error-handling] | `error` type, multi-value return     |
| Exception              | None                                         | `panic` and `recover` functions      |

### Types

|                  | Pen                                           | Go                             |
| ---------------- | --------------------------------------------- | ------------------------------ |
| Number           | `number` (IEEE 754)                           | `int`, `float64`, ...          |
| Sequence         | `[number]` (lazy list)                        | `[]int` (array or slice)       |
| Map              | `{string: number}`                            | `map[string]int`               |
| Optional value   | `none`, union types                           | null pointer (or _zero_ value) |
| Function         | `\(number, boolean) string`                   | `func(int, bool) string`       |
| Union            | `number \| string`                            | Interface                      |
| Top type         | `any`                                         | `any` (`interface{}`)          |
| Interface        | Records                                       | Interface                      |
| Futures          | Functions (thunks)                            | None                           |
| Concurrent queue | `[number]`, [built-in functions][concurrency] | `chan int`                     |

The `\` (lambda, λ) notation in function types and literals originates from other functional programming languages like [Haskell](https://haskell.org).

## Technical design

### Polymorphism without generics

Pen intentionally omit generics (or more specifically parametric polymorphism for functions and types) from its language features as well as the original [Go][go]. And it's one of the biggest experiments as most of existing functional languages have generics as their primary features.

Instead, we explore polymorphism with other language features, such as generic constructs (e.g. list comprehension and pattern matches,) subtyping, top types, reflection, code generation, and so on. A belief behind this decision is that Pen can achieve the same flexibility as other languages reducing complexity of the language itself. For the same reason, we don't adopt macros as we believe they are too powerful for humanity to handle.

### Dynamic effect system

Pen does not adopt any formal effect system of algebraic effects or monads. Instead, Pen rather uses a simple rule to manage side effects: all effects are passed down from the `main` functions to child functions. So unless we pass those impure functions to other functions explicitly, they are always pure. As such, Pen is an impure functional programming language although all runtime values are immutable. However, it still provides many of the same benefits purely functional languages do, such as determinicity and testability.

The reason we do not adopt any formal and statically provable effect system is to keep the language and its type system simple and lean for the purpose of improving developer productivity and software development scalability; we want to make Pen accessible and easy to learn for both newbie and expert programmers.

### Context switch

Like [Go][go], every function in Pen is suspendable and can be called asynchronously. This is realized by intermediate representation compiled into [Continuation Passing Style (CPS)](https://en.wikipedia.org/wiki/Continuation-passing_style) which also enables proper tail calls. Thus, Pen implements context switch without any platform-dependent codes for slight sacrifice of performance while Go requires logic written in assembly languages.

Currently, Pen does not use [delimited continuations](https://en.wikipedia.org/wiki/Delimited_continuation) for the following reasons.

- Traditional continuations are sufficient for our use cases, such as asynchronous programming.
- Delimited continuations require heap allocations although the second-class continuations do not.

### Reference counting GC

Pen implements [the Perceus reference counting][perceus] as its GC. Thanks to the state-of-the-art ownership-based RC algorithm, programs written in Pen performs much less than traditional RC where every data transfer or mutation requires counting operations. In addition, the algorithm reduces heap allocations significantly for records behind unique references, which brings practical performance without introducing unsafe mutability.

See also [How to Implement the Perceus Reference Counting Garbage Collection](https://dev.to/raviqqe/implementing-the-perceus-reference-counting-gc-5662).

### Inductive values

> TBD

### Stackful coroutines

> TBD

## Contributing

Pen is under heavy development. Feel free to post [Issues](https://github.com/pen-lang/pen/issues) and [Discussions](https://github.com/pen-lang/pen/discussions)!

### Workflows

#### Installing from source

See [Install](https://pen-lang.org/introduction/install#building-from-source).

#### Building crates

```sh
tools/build.sh
```

#### Running unit tests

```sh
tools/unit_test.sh
```

#### Running integration tests

```sh
tools/build.sh
tools/integration_test.sh
```

#### Running benchmarks

Those benchmarks include ones written in both Pen and Rust.

```sh
tools/benchmark.sh
```

#### Linting crates

```sh
tools/lint.sh
```

#### Formatting crates

```sh
tools/format.sh
```

### Directory structure

- [`cmd`](cmd): Commands
  - [`pen`](cmd/pen): `pen` command
- [`lib`](lib): Libraries for compiler, formatter, documentation generator, etc.
  - [`app`](lib/app): Platform-agnostic application logic for `pen` command
  - [`infra`](lib/infra): Platform-dependent logic for `pen` command
  - [`ast`](lib/ast): Abstract Syntax Tree (AST) types
  - [`hir`](lib/hir): High-level Intermediate Representation (HIR) types and semantics
  - [`mir`](lib/mir): Mid-level Intermediate Representation (MIR)
  - [`ast-hir`](lib/ast-hir): AST to HIR compiler
  - [`hir-mir`](lib/hir-mir): HIR to MIR compiler
  - [`mir-fmm`](lib/mir-fmm): MIR to [F--](https://github.com/raviqqe/fmm) compiler
- [`packages`](packages): Packages written in Pen
  - [`core`](packages/core): Package for platform-independent algorithms and data structures
  - [`os`](packages/os): Package for a common OS interface
- [`tools`](tools): Developer and CI tools
- [`doc`](doc): Documentation at [pen-lang.org](https://pen-lang.org/)

## License

Pen is dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

[concurrency]: https://pen-lang.org/guides/concurrency-and-parallelism
[error-handling]: https://pen-lang.org/references/language/syntax#error-handling
[gc]: #reference-counting-gc
[go]: https://go.dev/
[install]: https://pen-lang.org/introduction/install
[perceus]: https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/
[rust]: https://www.rust-lang.org/
