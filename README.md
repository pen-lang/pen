<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](https://github.com/pen-lang/pen/blob/main/LICENSE-APACHE)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is the programming language for **scalable** software development, focused on software maintainability and portability.

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
- [Language reference][syntax]
- Code examples
  - [Applications and libraries](https://github.com/pen-lang/pen/tree/main/examples)
  - [Snippets](https://pen-lang.org/examples)

## Vision

Pen aims to make large-scale software development efficient where a number of people develop software together for a long time. To realize that, it focuses on software **maintainability** and **portability**.

- Maintainability
  - Simplicity: The language is small and easy to learn but also full featured.
  - Testability: Unit tests are always fast and reliable.
  - Modifiability: Developers can change application logic independently from implementation details.
- Portability
  - Programs written in the language can be ported to different platforms including [WebAssembly](https://webassembly.org/).

## Features

### Minimal language

- Its [syntax][syntax] and [type system](https://pen-lang.org/references/language/types.html) are small, simple, and easy to learn.
- Yet, the language supports all the modern features.

### Concurrent/parallel computation

- The language and its runtime enables thread-safe concurrent/parallel computation.
- For more information, see [Concurrency and parallelism](https://pen-lang.org/guides/concurrency-and-parallelism.html).

### System packages

- [System packages](https://pen-lang.org/advanced-features/writing-system-packages.html) encapsulate side effects as separate packages.
- No other packages have side effects unless injected them into explicitly.

### Reliable testing

- Unit tests are always deterministic and fast.
- No flaky or slow tests bother developers.

### Even more...

- Static typing
- Immutable values
- Pure functions by default
- Errors as values
- Asynchronous I/O
- Cross compile
- [Rust](https://www.rust-lang.org/)/C Foreign Function Interface (FFI)

## License

Pen is dual-licensed under [MIT](https://github.com/pen-lang/pen/blob/main/LICENSE-MIT) and [Apache 2.0](https://github.com/pen-lang/pen/blob/main/LICENSE-APACHE).

[syntax]: https://pen-lang.org/references/language/syntax.html
