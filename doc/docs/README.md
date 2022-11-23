<p align="center"><img width="300px" src="icon.svg" /></p>

# Pen programming language

Pen is the parallel, concurrent, and functional programming language for [**scalable** software development](#vision), focused on software maintainability and portability.

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

## Vision

Pen aims to make large-scale software development efficient where many engineers develop software together for a long time. To realize that, it focuses on software **maintainability** and **portability**.

- Maintainability
  - Simplicity: The language is small and easy to learn yet full featured.
  - Testability: Tests are always fast and reliable.
  - Flexibility: Developers can change codes easily without regression.
- Portability
  - Programs written in Pen can be ported to different platforms including [WebAssembly](https://webassembly.org/).

## Features

### Minimal language

- Its [syntax][syntax] and [type system](references/language/types.md) are small, simple, and easy to learn.
- Yet, the language supports all the modern features.

### Concurrent/parallel computation

- The language and its runtime enables thread-safe concurrent/parallel computation.
- For more information, see [Concurrency and parallelism](guides/concurrency-and-parallelism.md).

### Reliable testing

- Tests are always deterministic and fast.
- Tests are side-effect free and independent from test environment.

### No built-in system library

- There is no built-in system library dependent on platforms.
- Developers choose [system packages][system-packages] suitable for their applications.
- [System packages][system-packages] encapsulate platform-dependent codes and side effects.
- No other kind of package causes side effects without explicit injection.

### Security

- [No runtime exception][error-handling]
- Memory safe
- No undefined behavior
- No data race

### Even more...

- Static typing
- Immutable values
- Pure functions by default
- [Errors as values][error-handling]
- Asynchronous I/O
- [Cross compile](advanced-features/cross-compile.md)
- [Rust](https://www.rust-lang.org/)/C Foreign Function Interface (FFI)

## License

Pen is dual-licensed under [MIT](https://github.com/pen-lang/pen/blob/main/LICENSE-MIT) and [Apache 2.0](https://github.com/pen-lang/pen/blob/main/LICENSE-APACHE).

[error-handling]: references/language/syntax.md#error-handling
[syntax]: references/language/syntax.md
[system-packages]: advanced-features/writing-system-packages.md
