# Building an executable

This page describes how to build an executable of a program written in Pen. It consists of the following steps:

1. Create an _application_ package.
1. Build the package and generate an executable.

## Creating an application package

[Application packages](/references/language/packages.md#application-packages) are packages compiled into executables.
To create it, you run a `pen create` command with your application's name (`foo` in the example below) in your terminal.

```sh
pen create foo
```

Then, you should see a `foo` directory under your current directory. When you go in there, you should see a `main.pen` source file and a `pen.json` file for [package configuration](/references/language/packages.md#package-configuration).

`main.pen`:

```pen
import Os'File

main = \(ctx context) none {
  _ = File'Write(ctx.Os, File'StdOut(), "Hello, world!\n")

  none
}
```

`pen.json`:

```
{
  "type": "application",
  "dependencies": {
    "Os": "pen:///os"
  }
}
```

The `main.pen` file contains a program that outputs a text, "Hello, world!" in a terminal. And the `pen.json` configuration file defines a package type of `application` and its dependencies. Here, it has only a dependency of the `Os` standard package.

## Building a package and generating an executable

To build the package, you run a `pen build` command in the package's directory.

```sh
pen build
```

Then, you will see an executable file named `app` in the directory. Run it to see your first "Hello, world!"

```sh
> ./app
Hello, world!
```

## Next steps

- [Creating a library](creating-a-library.md)
- [Syntax](/references/language/syntax.md)
