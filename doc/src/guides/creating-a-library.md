# Creating and using a library

This page describes how to create and use a library in Pen. It consists of the following steps:

1. Create a _library_ package.
1. Publish the library package.
1. Modify package configuration in a package where you want to use the library package.
1. Import functions and types from the library package.

## Creating a library package

[Library packages](/references/language/packages.md#library-packages) are packages imported and used by other packages.
To create it, you run a `pen create` command with your library's name (e.g. `foo`) in your terminal.

```sh
pen create foo
```

Then, you should see a `foo` directory under your current directory. When you go in there, you should see a `Foo.pen` source file and a `pen.json` file for [package configuration](/references/language/packages.md#package-configuration).

`Foo.pen`:

```pen
Add = \(x number, y number) number {
  x + y
}
```

`pen.json`:

```json
{
  "type": "library",
  "dependencies": {}
}
```

In this example, the `Foo.pen` file contains a function that adds two numbers. And the `pen.json` configuration file defines a package type of `library` and its dependencies of none.

## Publishing a library package

The easiest way to publish your library package is to push the package as a Git repository onto one of Git repository hosting services, such as [GitHub](https:://github.com).

```sh
git add .
git remote origin add https://github.com/your-name/foo
```

## Modifying package configuration

To use the library package, you need to add the package as a dependency in another package. To do so, after going into the different package, you modify its package configuration file with a library package's name and URL like the following. Note that you need to specify a `git` protocol scheme for library packages published as Git repositories.

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

- [Creating a library](creating-a-library.md)
- [Language syntax](/references/language/syntax.md)
