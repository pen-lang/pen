# Creating a library

This page describes how to create a library in Pen. It consists of the following steps:

1. Create a _library_ package.
1. Publish the package.

## Creating a library package

[Library packages](/references/language/packages.md#library-packages) are packages imported and used by other packages.
To create it, you run a `pen create --library` command with your library's name (e.g. `foo`) in your terminal.

```sh
pen create --library foo
```

Then, you should see a `foo` directory in your current directory. When you go there, you should see a `Foo.pen` source file and a `pen.json` file for [package configuration](/references/language/packages.md#package-configuration).

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
git push
```

## Next steps

- [Building an executable](building-an-executable.md)
- [Language syntax](/references/language/syntax.md)
