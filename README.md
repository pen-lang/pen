<p align="center"><img width="300px" src="https://pen-lang.org/favicon.svg" /></p>

# Pen programming language

[![GitHub Action](https://img.shields.io/github/workflow/status/pen-lang/pen/test?style=flat-square)](https://github.com/pen-lang/pen/actions)
[![License](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-yellow?style=flat-square)](https://github.com/pen-lang/pen#license)
[![Twitter](https://img.shields.io/badge/twitter-%40pen__language-blue?style=flat-square)](https://twitter.com/pen_language)

Pen is a functional descendant of the [Go][go] programming language focused on application programming.

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

## Difference from Go

### Types

| Type             | Go                             | Pen                         |
| ---------------- | ------------------------------ | --------------------------- |
| Number           | `int`, `float64`, ...          | `number` (IEEE 754)         |
| Sequence         | `[]int` (array or slice)       | `[number]` (lazy list)      |
| Map              | `map[string]int`               | `{string: number}`          |
| Concurrent queue | `chan int`                     | `[number]` (lazy list)      |
| Optional         | null pointer (or _zero_ value) | `none`                      |
| Function         | `func(int, bool) string`       | `\(number, boolean) string` |

## Technical details

> WIP

## Contributing

> WIP

## License

Pen is dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

[go]: https://go.dev/
[syntax]: https://pen-lang.org/references/language/syntax.html
