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
