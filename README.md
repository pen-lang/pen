<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](LICENSE.md)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is a statically typed functional programming language for application programming with [system injection](#system-injection). Its design is heavily inspired by [the Go programming language][go] and many functional programming languages like [Haskell](https://www.haskell.org/) and [Koka](https://koka-lang.github.io/koka/doc/index.html).

## Vision

Pen aims to make large-scale software development efficient where a number of people develop software together over a long time. To realize that, it focuses on software **maintainability** and **portability**.

- Maintainability
  - Everyone can learn the language and participate in actual development quickly.
  - Developers can focus on application logic rather than ever-changing implementation details.
  - The language keeps codes testable by injecting non-testable codes explicitly.
- Portability
  - Programs written in the language can be ported to different platforms including [WASM](https://webassembly.org/).

## Background

Simplicity enables efficient collaboration of developers. [The Go programming language][go] has been notably successful as it's been one of the most simple but practical programming languages ever made. That being said, [Go 2](https://go.dev/blog/go2-here-we-come) decided to compromise some complexity for its evolution, such as its [generics](https://github.com/golang/go/issues/43651) proposal.

On the other hand, Pen aims to be **even simpler by focusing only on application programming** as its target domain while adopting the same philosophy of simplicity. It pursues its minimal language design further after removing several features from Go like pointers, mutability, method syntax, global variables, circular references, etc.

## Features

### [Minimal language design][syntax]

- Its [syntax][syntax] and [type system](https://pen-lang.org/references/language/types.html) are made as small as possible.
- Even smaller than Go!

### Static typing

- Type inference
- Subtyping
- No generics

### Functional programming

- Immutable values
- Pure functions by default

### [System injection](https://pen-lang.org/advanced-features/system-injection.html)

- A mechanism to inject system functions explicitly into functions
- In other words, it's a dynamically typed effect system.

### Others

- Automatic memory management
  - Ownership-based reference counting
- [Stress-free error handling](https://pen-lang.org/references/language/syntax.html#error-handling)
- Cross compile
- Foreign Function Interface (FFI)
- Tail call optimization
- CPS transformation
- Deterministic testing framework (WIP)
- Asynchronous operations (WIP)
  - Based on continuations
- Thread-safe parallel computation (WIP)

## Install

You can install Pen via [Homebrew](https://brew.sh/) by running the following command in your terminal.

```sh
brew install pen-lang/pen/pen
```

## Documentation

[Here](https://pen-lang.org)

## Roadmap

Items are ordered by priority.

- [x] Basic syntax
- [x] CPS transformation
- [x] Capability-based effect system
- [x] Performant GC
  - [x] Automatic reference counting
- [x] Foreign function interface
- [x] Basic OS interface
- [x] WASM backend
- [x] Stream-based list type
- [ ] Testing framework
- [ ] Serialization / deserialization
- [ ] Map type
- [ ] Code formatter
- [ ] Asynchronous operations
- [ ] Parallel computation
- [ ] Full OS interface
  - [ ] TCP/UDP sockets
  - [ ] Process
- [ ] IDE/editor support
  - [ ] Language server
- [ ] Mutable state
  - [ ] Thread safety
- [ ] Web browser interface
- [ ] Binary support

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

[go]: https://golang.org
[syntax]: https://pen-lang.org/references/language/syntax.html
