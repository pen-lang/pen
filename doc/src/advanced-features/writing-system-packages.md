# Writing system packages

Using existing system packages covers most use cases in application development. However, by writing your own system packages, you can achieve the following:

- Define your own system interfaces of functions and types with side effects.
- Link applications in arbitrary file formats.

> Caveat: Providing bad system packages which do not conform to conventions can break the ecosystem of the language! In the worst cases, they might make applications malfunction. Please be careful to follow the conventions to keep applications maintainable and portable.

This page assumes that you have already read [Packages](/references/language/packages.md).

## Functionalities of system packages

System packages have the following three functionalities.

### Defining main function types

Every system package must have a module named `MainFunction.pen` in which a `MainFunction` function type is defined. Literally, the function type is used as a type of `main` functions in `main.pen` modules in application packages using the system package.

For example, a system package for command line applications might have the following `MainFunction.pen` module:

```pen
import 'Context { Context }

type MainFunction = \(Context) none | error
```

At the liking phase, actual main functions compiled from source codes are available as `_pen_main` in object files. And the object files are passed to system packages' [linking scripts](#linking-application-files) described later.

### Providing system functions and types

System packages are the only places where you can define functions that have side effects. Because they provide those system functions, applications can perform side effects, such as:

- I/O (console output, file system operations, etc.)
- Random number generation
- Concurrent/parallel computation

#### Conventions

**System packages should never expose side effects directly through their functions**; all exported functions of their APIs must be pure. Instead, every system package should provide (usually) one _context_ type on which those functions depend to make side effects. Then, actual context values of the type are injected into entry points of applications: the `main` functions.

For example, a system package for command line applications should have the following types and functions:

```pen
# Defines a foreign function to output a string on console.
import foreign "c" _pen_cli_put_string \(string) none

type Context {
  putString: _pen_cli_put_string,
  ...
}

Print = \(ctx Context, s string) none {
  ctx.putString(s)
}
```

rather than:

```pen
import foreign "c" _pen_cli_put_string \(string) none

Print = \(s string) none {
  _pen_cli_put_string(s)
}
```

### Linking application files

Each system package has a script file named `pen-link` at its top directory. On every build, the executable file is run with object files specified as command line arguments to link an application file. The script files may or may not have file extensions.

The scripts should accept the following command line arguments. Outputs of the scripts are discarded unless some errors occur during linking.

| Argument           | Required | Description                    |
| ------------------ | -------- | ------------------------------ |
| `-t <target>`      | No       | Target triple                  |
| `-o <application>` | Yes      | Path of an application file    |
| `<archive>...`     | Yes      | Paths of archive files to link |

## Examples

[The OS standard package](https://github.com/pen-lang/pen/tree/main/lib/os) is one of examples of system packages.
