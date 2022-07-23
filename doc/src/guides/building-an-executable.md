# Building an executable

This page describes how to build an executable of a program written in Pen. It consists of the following steps:

1. Create an _application_ package.
1. Build the package into an executable.

## Creating an application package

[Application packages](/references/language/packages.md#application-packages) are packages that are built into executables.
To create it, you run a `pen create` command with your application's name (e.g. `foo`) in your terminal.

```sh
pen create foo
```

Then, you should see a `foo` directory in your current directory. When you go there, you should see a `main.pen` source file and a `pen.json` file for [package configuration](/references/language/packages.md#package-configuration).

`main.pen`:

```pen
import Os'File

main = \(ctx context) none {
  _ = File'Write(ctx.Os, File'StdOut(), "Hello, world!\n")

  none
}
```

`pen.json`:

```json
{
  "type": "application",
  "dependencies": {
    "Os": "pen:///os"
  }
}
```

In this example, the `main.pen` file contains a program that outputs a text, "Hello, world!" in a terminal. And the `pen.json` configuration file defines a package type of `application` and its dependencies. Here, it has only a dependency of the `Os` [system package](/references/language/packages.md#system-packages).

## Building a package into an executable

To build the package, you run a `pen build` command in the package's directory.

```sh
pen build
```

Then, you will see an executable file named `app` in the directory. Now, you can run it to see its output, "Hello, world!"

```sh
./app # -> Hello, world!
```

## Next steps

- [Creating a library](creating-a-library.md)
- [Using a library](using-a-library.md)
- [Language syntax](/references/language/syntax.md)
