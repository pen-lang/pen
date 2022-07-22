<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](https://github.com/pen-lang/pen#license)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is a **functional** descendant of [the Go programming language][go] focused on **application programming**. It aims for further simplicity, testability, and portability to empower sustainable software development.

Pen also provides [Rust][rust]/C FFI to reuse existing resources written in those languages. Pen comes with no built-in system library or runtime and can compile platform-independent programs even for [WebAssembly](https://webassembly.org/).

```pen
import Os'Context { Context }
import Os'File
import Os'Process

sayHello = \(ctx Context) none | error {
  File'Write(ctx, File'StdOut(), "Hello, world!\n")?

  none
}

main = \(ctx context) none {
  e = sayHello(ctx.Os)

  if _ = e as none {
    none
  } else if error {
    Process'Exit(ctx.Os, 1)
  }
}
```

## Install

See [Install](https://pen-lang.org/introduction/install.html).

## Documentation

- [Getting started](https://pen-lang.org/introduction/getting-started.html)
- [Language reference](https://pen-lang.org/references/language/syntax.html)
- Code examples
  - [Applications and libraries](https://github.com/pen-lang/pen/tree/main/examples)
  - [Snippets](https://pen-lang.org/examples)

## Differences from [Go][go]

### Overview

|                      | Pen                              | Go                           |
| -------------------- | -------------------------------- | ---------------------------- |
| Primary domain       | Application programming          | System programming           |
| Paradigm             | Functional                       | Imperative / object-oriented |
| Memory management    | [Reference counting][gc]         | Concurrent mark-and-sweep    |
| System library       | Your choice!                     | Built-in                     |
| Values               | Immutable                        | Mutable                      |
| Data race prevention | Built into [GC][gc]              | Dynamic analysis             |
| Context switch       | [Continuations](#context-switch) | Platform dependent           |

### Types

|                  | Pen                            | Go                             |
| ---------------- | ------------------------------ | ------------------------------ |
| Number           | `number` (IEEE 754)            | `int`, `float64`, ...          |
| Sequence         | `[number]` (lazy list)         | `[]int` (array or slice)       |
| Map              | `{string: number}`             | `map[string]int`               |
| Concurrent queue | `[number]`, built-in functions | `chan int`                     |
| Optional value   | `none`                         | null pointer (or _zero_ value) |
| Function         | `\(number, boolean) string`    | `func(int, bool) string`       |
| Union            | `number \| string`             | Interface                      |
| Top type         | `any`                          | `any` (`interface{}`)          |
| Interface        | Records                        | Interface                      |

The `\` (lambda, λ) notation in function types and literals originates from other functional programming languages like [OCaml](https://ocaml.org) and [Haskell](https://haskell.org).

## Technical design

### Context switch

Like [Go][go], every function in Pen is suspendable and can be called asynchronously. This is realized by intermediate representation compiled into [Continuation passing style (CPS)](https://en.wikipedia.org/wiki/Continuation-passing_style) which also enables proper tail calls. Therefore, Pen does not need any platform-dependent codes for this while context switch in Go is written in assembly languages.

Currently, Pen does not use [delimited continuations](https://en.wikipedia.org/wiki/Delimited_continuation) for the following reasons.

- Traditional continuations are sufficient for our use cases, such as asynchronous programming.
- Delimited continuations require heap allocations although the second-class continuations do not.

### Reference counting GC

Pen implements [the Perceus reference counting][perceus] as its GC. Thanks to the state-of-the-art RC algorithm, programs written in Pen performs less than traditional reference counting where every data transfer or mutation requires counting operations. In addition, the algorithm reduces heap allocations significantly when records behind unique references, which brings practical performance without introducing unsafe mutability.

See also [How to Implement the Perceus Reference Counting Garbage Collection](https://hackernoon.com/how-to-implement-the-perceus-reference-counting-garbage-collection).

## Contributing

### Workflows

#### Installing from source

See [Install](https://pen-lang.org/introduction/install.html#building-from-source).

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

> WIP

## License

Pen is dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

[gc]: #reference-counting-gc
[go]: https://go.dev/
[perceus]: https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/
[rust]: https://www.rust-lang.org/
