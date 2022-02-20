<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](https://github.com/pen-lang/pen/blob/main/LICENSE.md)
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

See [Install](https://pen-lang.org/guides/install.html).

## Documentation

- [Getting started](https://pen-lang.org/guides/getting-started.html)
- [Language reference][syntax]
- Code examples
  - [Applications and libraries](https://github.com/pen-lang/pen/tree/main/examples)
  - [Snippets](https://pen-lang.org/examples)

## Vision

Pen aims to make large-scale software development efficient where a number of people develop software together for a long time. To realize that, it focuses on software **maintainability** and **portability**.

- Maintainability
  - Simplicity: The language is small and easy to learn but also full featured.
  - Testability: Unit tests are always fast and reliable.
  - Modifiability: Developers can change application logic without changing implementation details, and vice versa.
- Portability
  - Programs written in the language can be ported to different platforms including [WebAssembly](https://webassembly.org/).

## Features

### Minimal language

- Its [syntax][syntax] and [type system](https://pen-lang.org/references/language/types.html) are small, simple, and easy to learn.
- Yet, the language supports all the modern features.

### Concurrent/parallel computation

The language and its runtime enables thread-safe concurrent/parallel computation.

### System injection

- [System injection](https://pen-lang.org/advanced-features/system-injection.html) is a novel mechanism to isolate application logic from implementation details.
- The language injects system functions into applications explicitly.
- Developers can even define their own system functions.

### Deterministic testing

- All unit tests are deterministic.
- Therefore, testing is always reliable and fast.
- You never get bothered by flaky or slow tests.

### Even more...

- Static typing
- Immutable values
- Pure functions by default
- Errors as values 
- Asynchronous I/O
- Cross compile
- [Rust](https://www.rust-lang.org/) foreign function interface (FFI)

## License

Pen is released under open source licenses. See [its LICENSE file](https://github.com/pen-lang/pen/blob/main/LICENSE.md) for more information.

[syntax]: https://pen-lang.org/references/language/syntax.html
