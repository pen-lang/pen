# Pen

The programming language for scalable software engineering

## Vision

Pen is designed for software engineering, where we need to change software and provide values to users continuously over time. Particularly, it's optimized for the one by a large number of people and/or over a long time.

To make such engineering efficient, it focuses on:

- Maintainability
  - Everyone can learn the language and participate in actual development quickly.
  - People having different backgrounds can collaborate with each other at a minimal cost.
- Portability
  - [System injection](#system-injection) isolates application logic from implementation details in order to bring long expectancy of your software as well as maintainability.
  - Its C/[Rust](https://www.rust-lang.org/) Foreign Function Interface (FFI) provides interoperability with other languages.

## Features

### Minimal design

- Its syntax and type system are simple and easy to learn.
- Its minimal language features keep codes maintainable.

### System injection

- System APIs are always injected as arguments to main functions.
- Developers can define their own system APIs and main function types.

### Even more...

- Static typing
  - Data types are checked at compile time so that developers can catch errors earlier.
- Immutable values
  - Values are immutable, which leads to predictable and testable codes.
- Pure functions by default
  - Functions are pure; they work just like math functions unless developers inject side effects explicitly.
- Errors as values
  - Errors are merely data. Its special syntax brings a convenient way to handle errors inside each function.
- Cross compile
  - The compiler and runtime support different CPU architectures, operating systems, web browsers and [WASI](https://wasi.dev/) (WIP.)
- Asynchronous operation (WIP)
  - Every function is possibly asynchronous while called in the same way as synchronous ones.
- Parallel computation (WIP)
  - The runtime and library provide tools for thread-safe parallel computation that leverage multi-core CPUs.

## License

[Apache 2.0](https://github.com/pen-lang/pen/blob/main/LICENSE)
