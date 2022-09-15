# Writing system packages

Using existing system packages covers most use cases in application development. However, by writing your own system packages, you can achieve the following:

- Define your own system interfaces of functions and types with side effects.
- Link applications in arbitrary file formats.

This page assumes that you have already read [Packages](../references/language/packages.md).

> Caveat: Providing bad system packages which do not conform to conventions can break the ecosystem of the language! In the worst cases, they might make applications malfunction. Please be careful to follow the conventions to keep applications maintainable and portable.

## Functionalities of system packages

System packages have the following functionalities:

- Define context types.
- Provide system interfaces as functions and types.
- Link application files.

### Defining context types

Every system package must have a module named `Context` at the top level. The module defines a `Context` type and an `UnsafeNew` function that returns a `Context` value with no argument.

For example, a system package for command line applications might have the following `Context` module:

```pen
...

type Context {
  print \(string) none
}

UnsafeNew = \() Context {
  Context{
    print: ...
  }
}
```

The language's compiler uses these type and function to compose a `context` type passed to `main` functions in `main` modules in application packages.

### Providing system functions and types

System packages are the only places where you can define functions that have side effects. Thanks to system packages, applications can perform effects to interact with the world, such as:

- Console input/output
- File system operations
- Networking
- Random number generation

Note that **system packages should never expose side effects directly through their functions**; all public functions in system packages must be purely functional. Instead, you need to pass a `Context` type to every effect-ful function for it to make side effects.

For example, a system package for command line applications might have the following types and functions:

```pen
# Define a foreign function to output a string in console.
import foreign _pen_cli_print \(string) none

type Context {
  print: _pen_cli_print,
}

Print = \(ctx Context, s string) none {
  ctx.print(s)
}
```

rather than:

```pen
import foreign _pen_cli_print \(string) none

Print = \(s string) none {
  # Oh, no! We make side effects in a public function directly.
  _pen_cli_print(s)
}
```

### Linking application files (optional)

System packages might have optional script files named `pen-link` at their top directories. On every build of application packages using the system packages, the script files run given object files specified as command line arguments to link application files. The script files may or may not have file extensions.

The scripts should accept the following command line arguments.

| Argument           | Required | Description                                                    |
| ------------------ | -------- | -------------------------------------------------------------- |
| `-t <target>`      | No       | Target triple                                                  |
| `-o <application>` | Yes      | Path of an application file                                    |
| `<archive>...`     | Yes      | Paths of archive files sorted topologically from main packages |

At the liking phase, compiled main functions are available under a symbol named `_pen_main` with Pen's native calling convention.

## Examples

[The `Os` standard package](https://github.com/pen-lang/pen/tree/main/packages/os) is an example of system packages.
