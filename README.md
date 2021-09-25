<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](https://github.com/pen-lang/pen/blob/main/LICENSE.md)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is the programming language that makes software development **scalable**, focusing on software maintainability and portability.

```pen
import System'Context { Context }
import System'File

sayHello = \(ctx Context) none | error {
  File'Write(ctx, File'StdOut(), "Hello, world!\n")?

  none
}

main = \(ctx Context) number {
  e = sayHello(ctx)

  if e = e as none {
    0
  } else if error {
    1
  }
}
```

## Install

See [Install](https://pen-lang.org/guides/install.html).

## Documentation

- [Getting started](https://pen-lang.org/guides/getting-started.html)
- [Language reference](https://pen-lang.org/references/language/syntax.html)
- [Code examples](https://pen-lang.org/examples/standard-packages/os.html)

## Vision

Pen aims to make large-scale software development efficient where a number of people develop software together. To realize that, it focuses on software **maintainability** and **portability**.

- Maintainability
  - Simplicity: The language is small and easy to learn but also full featured.
  - Testability: Unit tests are always fast, reliable, and independent with each other.
  - Modifiability: Developers can change application logic without doing implementation details, and vice versa.
- Portability
  - Programs written in the language can be ported to different platforms including [WASM](https://webassembly.org/).

## Features

### Minimal language design

- [Syntax][syntax] and [type system](https://pen-lang.org/references/language/types.html) are made as small as possible.
- Yet, the language supports all the modern features.
  - Functional programming
  - Effect system
  - Dependency injection
  - Asynchronous operations
  - Parallel computation

### [System injection](https://pen-lang.org/advanced-features/system-injection.html)

- System functions are always injected as arguments of main functions.
- That isolates and protects application logic from implementation details for both maintainability and portability.
- Developers can define their own system functions and build applications on top of them.

### Others

- Static typing
- Immutable values
- Pure functions by default
- Errors as values
- Cross compile
- [Rust](https://www.rust-lang.org/)/C Foreign Function Interface

### Work in progress...

#### Deterministic tests (WIP)

Unit tests are deterministic to enable reliable continuous integration.

#### Asynchronous operation (WIP)

Functions can be called asynchronously to run multiple tasks concurrently.

#### Parallel computation (WIP)

The runtime and library provide tools for thread-safe parallel computation that leverage multi-core CPUs.

## License

Pen is released under open source licenses. See [LICENSE](https://github.com/pen-lang/pen/blob/main/LICENSE.md) for more information.

[go]: https://golang.org
[syntax]: https://pen-lang.org/references/language/syntax.html
