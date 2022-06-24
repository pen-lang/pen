# Coding style

This page describes the common coding style used in programs written in Pen.

## Spacing

Use 2 space characters for indentation.

```pen
Foo = \(x number) number {
  if x == 0 {
    "Succeeded!"
  } else {
    "Failed..."
  }
}
```

## Naming convention

Naming is important to keep codes consistent. The language currently has the following naming conventions.

| Kind               | Case style       | Examples                     |
| ------------------ | ---------------- | ---------------------------- |
| Variables          | Camel case       | `fooBar`, `FooBar`, `i`, `x` |
| Functions          | Camel case       | `fooBar`, `FooBar`, `f`, `g` |
| Types              | Camel case       | `fooBar`, `FooBar`           |
| Modules            | Camel case       | `fooBar`, `FooBar`           |
| Module directories | Camel case       | `fooBar`, `FooBar`           |
| Packages           | Upper camel case | `FooBar`                     |

### Global and local names

You should use descriptive names for global functions and types. But, on the other hand, you are encouraged to use abbreviated names for local variables as long as that doesn't incur ambiguity. For example, you might use the following abbreviated names:

- `i` for `index`
- `c` for `requestCount`
- `sys` for `system`
- `ctx` for `context`

### Acronyms

Acronyms are treated as single words.

- `Cpu`
- `Ast`
