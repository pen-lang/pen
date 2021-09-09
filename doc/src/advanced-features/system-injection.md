# System injection

> Caveat: Providing bad system packages which do not conform to conventions can break the ecosystem of the language. Please be careful to follow them and keep applications maintainable and portable!

Using system injection, you can define your own system APIs and build applications in file formats of your choice. Those system APIs and application linking logic are bundled into [system packages](#system-packages).

## System packages

In [their package configurations](/references/language/packages.md#package-configuration), [application packages](/references/language/packages.md#kinds-of-packages) need to specify system packages that are special kinds of library packages and define system APIs for the programs to interact with the world.

System packages do the following three things.

### Defining main function types

A system package must have a module named `MainFunction.pen` in which a `MainFunction` function type is defined. Literally, the function type is used as a type of `main` functions in `Main.pen` modules in application packages.

For example, a system package for command line applications might have the following `MainFunction.pen` module:

```pen
import 'Context

type MainFunction = \(ctx Context'Context) none | error
```

### Providing system APIs

System packages are the only places where we can define functions that have side effects. As they provide those system functions, applications can perform I/O, such as console output and file system operations, to make actual effects to the world.

#### Conventions

**System packages should never expose side effects directly through functions**; all exported functions of their APIs must be pure. Instead, every system package should provide (usually) one _context_ type on which those functions depend to make side effects.

For example, a system package for command line applications might have the following API:

```pen
import foreign "c" _pen_put_string \(string) none

type Context {
  putString: _pen_put_string,
  ...
}

Print = \(ctx Context, s string) none | error {
  ctx.putString(s)
}
```

rather than:

```pen
import foreign "c" _pen_put_string \(string) none

Print = \(s string) none | error {
  _pen_put_string(s)
}
```

### Linking application files

Each system package has a script file named `pen-link` at its top directory. The executable file is run with object files specified by command line arguments to build and link an application file on every build. The script files may or may not have file extensions.

The scripts should accept the following command line arguments. Outputs of the scripts are simply discarded unless some errors occur during runs.

| Argument           | Required | Description                          |
| ------------------ | -------- | ------------------------------------ |
| `-t <target>`      | No       | Target triple                        |
| `-o <application>` | Yes      | Path of an application file to build |
| `<archive>...`     | Yes      | Paths of archive files to link       |

## Examples

[The OS standard package](https://github.com/pen-lang/pen/tree/main/lib/os) is one of examples of system packages.
