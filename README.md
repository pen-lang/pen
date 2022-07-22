<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](#license)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is a **functional** programming language focused on **application programming** following [Go][go]'s philosophy. It aims for further simplicity, testability, and portability to empower team (v. individual) and/or long-term (v. short-term) productivity.

Pen's system libraries and runtime are detachable from applications and it can compile the same applications even for [WebAssembly](https://webassembly.org/) and [WASI](https://wasi.dev/). Pen also provides [Rust][rust]/C FFI to reuse existing resources written in those languages.

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

Pen is available via [Homebrew](https://brew.sh/).

```sh
brew install pen-lang/pen/pen
```

For more information, see [Install](https://pen-lang.org/introduction/install.html).

## Examples

See [the `examples` directory](examples).

## Documentation

- [Getting started](https://pen-lang.org/introduction/getting-started.html)
- [Language reference](https://pen-lang.org/references/language/syntax.html)
- Code examples
  - [Applications and libraries](https://github.com/pen-lang/pen/tree/main/examples)
  - [Snippets](https://pen-lang.org/examples)

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
| Concurrent queue | `[number]`, [built-in functions][concurrency] | `chan int`                     |
| Optional value   | `none`, union types                           | null pointer (or _zero_ value) |
| Function         | `\(number, boolean) string`                   | `func(int, bool) string`       |
| Union            | `number \| string`                            | Interface                      |
| Top type         | `any`                                         | `any` (`interface{}`)          |
| Interface        | Records                                       | Interface                      |

The `\` (lambda, λ) notation in function types and literals originates from other functional programming languages like [Haskell](https://haskell.org).

## Technical design

### Context switch

Like [Go][go], every function in Pen is suspendable and can be called asynchronously. This is realized by intermediate representation compiled into [Continuation Passing Style (CPS)](https://en.wikipedia.org/wiki/Continuation-passing_style) which also enables proper tail calls. Thus, Pen implements context switch without any platform-dependent codes for slight sacrifice of performance while Go requires logic written in assembly languages.

Currently, Pen does not use [delimited continuations](https://en.wikipedia.org/wiki/Delimited_continuation) for the following reasons.

- Traditional continuations are sufficient for our use cases, such as asynchronous programming.
- Delimited continuations require heap allocations although the second-class continuations do not.

### Reference counting GC

Pen implements [the Perceus reference counting][perceus] as its GC. Thanks to the state-of-the-art RC algorithm, programs written in Pen performs less than traditional reference counting where every data transfer or mutation requires counting operations. In addition, the algorithm reduces heap allocations significantly when records behind unique references, which brings practical performance without introducing unsafe mutability.

See also [How to Implement the Perceus Reference Counting Garbage Collection](https://hackernoon.com/how-to-implement-the-perceus-reference-counting-garbage-collection).

## Contributing

Pen is under heavy development. Feel free to post [Issues](https://github.com/pen-lang/pen/issues) and [Discussions](https://github.com/pen-lang/pen/discussions)!

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

[concurrency]: https://pen-lang.org/guides/concurrency-and-parallelism.html
[error-handling]: https://pen-lang.org/references/language/syntax.html#error-handling
[gc]: #reference-counting-gc
[go]: https://go.dev/
[perceus]: https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/
[rust]: https://www.rust-lang.org/
