<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](LICENSE.md)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is a statically typed, strictly evaluated, functional programming language heavily inspired by [the Go programming language][go]. It aims to make large-scale software development efficient where a large number of people can develop software together over a long time.

To realize that, it focuses on software **maintainability** and **portability**.

## Background

[The Go programming language][go] has been notably successful since it emerged on 2010 while it's been one of the most simple but practical languages ever made. It has proved importance of simplicity for development scalability and the cost of language features that incur complexity and inconsistency at a large scale.

Although [Go 2](https://go.dev/blog/go2-here-we-come) has decided to compromise increased complexity for additional features like [generics](https://github.com/golang/go/issues/43651),

## Features

- Minimal language design
  - It's even smaller than [Go][go]!
- Static typing
- Functional programming
- Immutable values
- Pure functions by default
- [Side effect injection](https://pen-lang.org/advanced-features/system-injection.html)

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
- [ ] IDE/editor support
  - [ ] Language server
- [ ] Asynchronous operations
- [ ] Parallel computation
- [ ] Full OS interface
  - [ ] TCP/UDP sockets
  - [ ] Process
- [ ] Mutable state
  - [ ] Thread safety
- [ ] Web browser interface
- [ ] Binary support
- [ ] Self-hosting

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

[go]: https://golang.org
