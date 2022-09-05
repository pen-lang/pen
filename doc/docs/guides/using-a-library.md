# Using a library

This page describes how to use a library in Pen. It consists of the following steps:

1. Add a library package as a dependency in another package.
1. Import functions and types from the library package.

## Modifying package configuration

To use a library package, you need to add the package as a dependency in another package. To add the dependency, you modify a `pen.json` configuration file in the package adding the library package's name (e.g. `Foo`) and URL (e.g. `git://github.com/your-name/foo`) in a `dependencies` field like the following example. Note that you need to specify a `git` protocol scheme for library packages published as Git repositories. For other kinds of library packages, see [Package configuration](/references/language/packages.md#package-configuration).

```json
{
  "type": "application", // This can be any type.
  "dependencies": {
    "Foo": "git://github.com/your-name/foo"
  }
}
```

## Importing functions and types from a library package

To import functions and types from the library package, you use `import` statements in a source file of your module with a name of the library package (e.g. `Foo`) and a module name (e.g. `Math`) where functions or types you want to use are defined.

```pen
import Foo'Math
```

Then, you are ready to use those functions and types with a prefix of the module name! For example, to call a function named `Add` in the `Math` module, you can write `Math'Add(x, y)`.

```pen
type MyType = Math'Order

MyFunction = \(x number, y number) number {
  Math'Add(x, y)
}
```

## Next steps

- [Building an executable](building-an-executable)
- [Creating a library](creating-a-library)
- [Language syntax](/references/language/syntax)
  - [Import statement](/references/language/syntax.md#import-statement)
