# Pen

The functional programming language for scalable development

## Goals

Pen is designed for software development by a large number of people and/or over a long period of time.

To make such development efficient, it focuses on:

- Simplicity
  - Its syntax and type system are simple and easy to learn.
  - Its minimal language features keep codes maintainable.
- Portability
  - [System injection](#system-injection) isolates application logic from implementation details in order to bring long expectancy of your software.
  - Cross build supports many platforms of different CPU architectures, operating systems, web browsers and [WASI](https://wasi.dev/) (WIP.)
  - Its C/[Rust](https://www.rust-lang.org/) Foreign Function Interface (FFI) brings interoperability with other languages.

## Features

### System injection

- System APIs are always injected as arguments to main functions.
- Developers can define their own system APIs and main function types.

### Others

- Static typing
  - Data types are checked at compile time so that developers catch errors earlier.
- Immutable values
  - Values are immutable, which leads to more predictable and testable codes.
- Pure functions by default
  - Functions are pure; they work just like math functions unless developers inject side effects explicitly.
- Errors as values
  - Errors are merely data. Its special syntax brings a convenient way to handle errors inside each function.
- Asynchronous operation (WIP)
  - Every function is possibly asynchronous while called in the same way as synchronous ones.
- Parallel computation (WIP)
  - The runtime and library provides tools for thread-safe parallel computation that leverage multi-core CPUs.

## License

[Apache 2.0](https://github.com/pen-lang/pen/blob/main/LICENSE)
