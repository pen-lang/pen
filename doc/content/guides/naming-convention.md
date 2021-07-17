# Naming convention

| Kind               | Case style       | Examples                      |
| ------------------ | ---------------- | ----------------------------- |
| Variables          | Lower camel case | `fooBar`, `i`, `x`            |
| Functions          | Lower camel case | `fooBar`, `f`, `g`            |
| Types              | Upper camel case | `FooBar`                      |
| Modules            | Upper camel case | `FooBar`                      |
| Module directories | Upper camel case | `FooBar`                      |
| Packages           | Kebab case       | `github.com/foo-bar/baz-blah` |

## Global and local names

- Use descriptive names for global variables and functions.
- Use abbreviated names for local ones.
  - `i` for `index`
  - `c` for `requestCount`
  - `sys` for `system`

## Acronyms

Acronyms are treated as single words.

- `Cpu`
- `Ast`
