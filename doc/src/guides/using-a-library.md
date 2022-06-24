# Using a library

This page describes how to use a library in Pen. It consists of the following steps:

1. Add a library package as a dependency in another package.
1. Import functions and types from the library package.

## Modifying package configuration

To use a library package, you need to add the package as a dependency of another package. To do so, you modify a package configuration file adding a library package's name and URL in a `dependencies` field like the following example. Note that you need to specify a `git` protocol scheme for library packages published as Git repositories. For other kinds of library packages, see [Package configuration](/references/language/packages.md#package-configuration).

```jsonc
{
  "type": "application", // This can be also `library`.
  "dependencies": {
    "Foo": "git://github.com/your-name/foo"
  }
}
```

## Importing functions and types from a library package

> WIP

## Next steps

- [Building an executable](building-an-executable.md)
- [Creating a library](creating-a-library.md)
- [Language syntax](/references/language/syntax.md)
