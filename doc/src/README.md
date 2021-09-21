<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen

The programming language for scalable development

## Vision

Pen is designed for large-scale software development; it empowers a number of people to develop software together over a long time. To make such development efficient, it focuses on:

- Maintainability
  - Everyone can learn the language and participate in actual development quickly.
  - Developers can focus on application logic rather than ever-changing implementation details.
  - The language keeps codes testable by injecting non-testable codes explicitly.
- Portability
  - Programs written in the language can be ported to different platforms.

## Features

### Minimal design

- Its syntax and type system are simple and easy to learn.
- Its minimal language features keep codes comprehensive and consistent.

### [System injection](/advanced-features/system-injection.md)

- System functions are always injected through entry points of applications.
- That isolates and protects application logic from implementation details for both maintainability and portability.
- Developers can define their own system functions and build applications on top of them.

### Even more...

#### Static typing

Data types are checked at compile time so that developers can catch errors earlier.

#### Immutable values

All values are immutable, which leads to predictable and testable codes.

#### Pure functions by default

Functions are pure; they work just like math functions unless developers inject side effects explicitly.

#### Errors as values

Errors are merely data. Its special syntax provides a convenient way to handle errors.

#### Cross compile

The compiler and runtime support different CPU architectures, operating systems, web browsers and [WASI](https://wasi.dev/).

#### Foreign Function Interface (FFI)

Its [Rust](https://www.rust-lang.org/)/C FFI provides interoperability with other languages.

#### Deterministic tests (WIP)

Unit tests are deterministic to enable reliable continuous integration.

#### Asynchronous operation (WIP)

Functions can be called asynchronously to run multiple tasks concurrently.

#### Parallel computation (WIP)

The runtime and library provide tools for thread-safe parallel computation that leverage multi-core CPUs.

## License

Dual-licensed under [MIT](https://github.com/pen-lang/pen/blob/main/LICENSE-MIT) and [Apache 2.0](https://github.com/pen-lang/pen/blob/main/LICENSE-APACHE).
