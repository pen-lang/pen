# Getting started

## Install

See [Install](install).

## Creating a package

To create your first package, run the following command.

```sh
pen create foo
```

Then, you should see a `foo` directory under the current directory. When you go to the `foo` directory, you should see a `Main.pen` source file and a `pen.json` package configuration file there.

## Building a package

To build the package, run the following command in the `foo` directory.

```sh
pen build
```

Then, you will see an executable file named `app` in the directory. Run it to see your first "Hello, world!"

```sh
./app
```

## For more information...

Now, you can start editing `.pen` files and build your own application!

- To know more about the language components, see [Syntax](/references/language/syntax.md) and [Types](/references/language/types.md).
- To know how to use the standard packages, see [Standard packages](/references/standard-packages).
- To know how to add and use more modules in your package, see [Modules](/references/language/modules.md).
- To know how to import other packages, see [Packages](/references/language/packages.md).
