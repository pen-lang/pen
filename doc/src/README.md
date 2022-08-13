<p align="center"><img width="300px" src="/favicon.svg" /></p>

# Pen programming language

Pen is the parallel, concurrent, and functional programming language for [**scalable** software development](#vision), focused on software maintainability and portability.

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

## Vision

Pen aims to make large-scale software development efficient where many engineers develop software together for a long time. To realize that, it focuses on software **maintainability** and **portability**.

- Maintainability
  - Simplicity: The language is small and easy to learn yet full featured.
  - Testability: Tests are always fast and reliable.
  - Flexibility: Developers can change codes easily without introducing bugs.
- Portability
  - Programs written in Pen can be ported to different platforms including [WebAssembly](https://webassembly.org/).

## Features

### Minimal language

- Its [syntax][syntax] and [type system](/references/language/types.md) are small, simple, and easy to learn.
- Yet, the language supports all the modern features.

### Concurrent/parallel computation

- The language and its runtime enables thread-safe concurrent/parallel computation.
- For more information, see [Concurrency and parallelism](/guides/concurrency-and-parallelism.md).

### Reliable testing

- Unit tests are always deterministic and fast.
- No flaky or slow tests bother developers.

### No standard system library

- There is no default platform-dependent system library.
- Developers choose [system packages][system-packages] suitable for their applications.
- [System packages][system-packages] encapsulate platform-dependent codes and side effects.
- No package causes side effects without explicit injection.

### Security

- [No runtime exception][error-handling]
- No undefined behavior
- No data race

### Even more...

- Static typing
- Immutable values
- Pure functions by default
- [Errors as values][error-handling]
- Asynchronous I/O
- [Cross compile](/advanced-features/cross-compile.md)
- [Rust](https://www.rust-lang.org/)/C Foreign Function Interface (FFI)

## License

Pen is dual-licensed under [MIT](https://github.com/pen-lang/pen/blob/main/LICENSE-MIT) and [Apache 2.0](https://github.com/pen-lang/pen/blob/main/LICENSE-APACHE).

[error-handling]: /references/language/syntax.md#error-handling
[syntax]: /references/language/syntax.md
[system-packages]: /advanced-features/writing-system-packages.md
