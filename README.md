<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](https://github.com/pen-lang/pen#license)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is a **functional** variant of the [Go][go] programming language focused on application programming. It aims for further simplicity, testability, and portability to empower sustainable software development.

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

## [Documentation](https://pen-lang.org)

- [Getting started](https://pen-lang.org/introduction/getting-started.html)
- [Language reference][syntax]
- Code examples
  - [Applications and libraries](https://github.com/pen-lang/pen/tree/main/examples)
  - [Snippets](https://pen-lang.org/examples)

## Differences from Go

### Overview

|                      | Pen                     | Go                           |
| -------------------- | ----------------------- | ---------------------------- |
| Primary domain       | Application programming | System programming           |
| Paradigm             | Functional              | Imperative / object-oriented |
| Memory management    | Reference counting      | Concurrent mark-and-sweep    |
| System library       | Your choice!            | Built-in                     |
| Runtime              | None                    | Large                        |
| Values               | Immutable               | Mutable                      |
| Data race prevention | Built into GC           | Dynamic analysis             |
| Context switch       | [Continuations][cps]    | Architecture-dependent codes |

### Types

|                  | Pen                         | Go                             |
| ---------------- | --------------------------- | ------------------------------ |
| Number           | `number` (IEEE 754)         | `int`, `float64`, ...          |
| Sequence         | `[number]` (lazy list)      | `[]int` (array or slice)       |
| Map              | `{string: number}`          | `map[string]int`               |
| Concurrent queue | `[number]` (lazy list)      | `chan int`                     |
| Optional value   | `none`                      | null pointer (or _zero_ value) |
| Function         | `\(number, boolean) string` | `func(int, bool) string`       |
| Union            | `number \| string`          | Interface                      |
| Top type         | `any`                       | `any` (`interface{}`)          |
| Interface        | Records                     | Interface                      |

The `\` (lambda, λ) notation in function types and literals originates from other functional programming languages like [OCaml](https://ocaml.org) and [Haskell](https://haskell.org).

## Technical details

> WIP

## Contributing

> WIP

## License

Pen is dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

[cps]: https://en.wikipedia.org/wiki/Continuation-passing_style
[go]: https://go.dev/
[syntax]: https://pen-lang.org/references/language/syntax.html
