# Getting started

## Install

See [Install](install.md).

## Creating a package

To create your first package, run the following command.

```sh
pen create foo
```

Then, you should see a `foo` directory under the current directory. When you go to the `foo` directory, you should see a `main.pen` source file and a `pen.json` file for [package configuration](/references/language/packages.md#package-configuration).

## Building a package

To build the package, run the following command in the `foo` directory.

```sh
pen build
```

Then, you will see an executable file named `app` in the directory. Run it to see your first "Hello, world!"

```sh
./app
```

## Next steps

Now, you can start editing `*.pen` files and build your own application!

- For more code examples, see [Examples](/examples).
- To know more about the language's constructs, see [Syntax](/references/language/syntax.md) and [Types](/references/language/types.md).
- To know how to use the standard packages, see [Standard packages](/references/standard-packages).
- To know how to add more modules in your package, see [Modules](/references/language/modules.md).
- To know how to import other packages, see [Packages](/references/language/packages.md).
