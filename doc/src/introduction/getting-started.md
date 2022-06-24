# Getting started

## Install

See [Install](install.md).

## Creating a package

To create your first package, run a `pen create` command with your package's name in your terminal.

```sh
pen create foo
```

Then, you should see a directory named `foo` in your current directory. When you go into the directory, you should see a `main.pen` source file and a `pen.json` file for [package configuration](/references/language/packages.md#package-configuration).

## Building a package

To build the package, run a `pen build` command in the `foo` directory.

```sh
pen build
```

Then, you will see an executable file named `app` in the directory. Run it to see your first "Hello, world!"

```sh
./app # -> Hello, world!
```

Now, you can start editing source files and build your own application in Pen!

## Next steps

- To use a library package in your package, see [Using a library](/guides/using-a-library.md).
- For more code examples, see [Examples](/examples).
- For the language syntax, see [Syntax](/references/language/syntax.md) and [Types](/references/language/types.md).
- For usage of the standard packages, see [Standard packages](/references/standard-packages).
- To add more modules in your package, see [Modules](/references/language/modules.md).
