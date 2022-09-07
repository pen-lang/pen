# Command line tools

The `pen` command has the following sub-commands.

For more information, see its help message by running `pen --help`.

## `build` command

It builds a package in the current directory.

```sh
pen build
```

## `create` command

It creates a package of a given kind in a specified directory.

### Creating an application package

```sh
pen create foo
```

### Creating a library package

```sh
pen create --library foo
```

## `test` command

It runs test modules in a package. See also [Testing](../guides/testing.md).

```sh
pen test
```

## `format` command

It formats all module files in a package.

```sh
pen format
```

## `document` command

It generates a documentation file of a package and emits it to stdout.

```sh
pen document \
  --name Foo \
  --description "A package to provide Foo" \
  --url git://github.com/foo/foo
```
