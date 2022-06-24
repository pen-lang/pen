# Using a library

This page describes how to use a library in Pen. It consists of the following steps:

1. Add a library package as a dependency in a package.
1. Import functions and types from an added library package.

## Modifying package configuration

To use a library package, you need to add the package as a dependency of a package where you want to use the library package. To do so, you modify a package configuration file with a library package's name and URL like the following. Note that you need to specify a `git` protocol scheme for library packages published as Git repositories.

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
- [Language syntax](/references/language/syntax.md)
