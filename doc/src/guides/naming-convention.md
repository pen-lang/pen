# Naming convention

| Kind               | Case style       | Examples                     |
| ------------------ | ---------------- | ---------------------------- |
| Variables          | Camel case       | `fooBar`, `FooBar`, `i`, `x` |
| Functions          | Camel case       | `fooBar`, `FooBar`, `f`, `g` |
| Types              | Camel case       | `fooBar`, `FooBar`           |
| Modules            | Upper camel case | `FooBar`                     |
| Module directories | Upper camel case | `FooBar`                     |
| Packages           | Upper camel case | `FooBar`                     |

## Global and local names

- Use descriptive names for top-level functions and types.
- Use abbreviated names for function-local ones.
  - `i` for `index`
  - `c` for `requestCount`
  - `sys` for `system`
  - `ctx` for `context`

## Acronyms

Acronyms are treated as single words.

- `Cpu`
- `Ast`
