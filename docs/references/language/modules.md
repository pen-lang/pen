# Modules

## Overview

- Each source file composes a module.
- Modules contain their functions and types.
- They are exported to and imported from other modules.

## Exporting functions and types from a module

Name functions and types in an upper camel case.

```
type Foo {
  ...
}

type Bar = ...

Foo = \() number {
  ...
}
```

## Importing functions and types from a module

First, place an `import` statement to import a module at the top of a module you want to import them into.

The first component of a path in the statement is a name of an external package you declare in [a `pen.json` file](../packages#package-configuration) (`Foo`.) It is omitted if the imported module is in the same package. The rest of the path components are directory names where a module exists (`Bar`) and the basename of the module filename (`Baz`.)

```
import Foo'Bar'Baz
```

Then, you can access exported members of the module with its prefix.

```
type Foo = Baz'Foo

bar = \(x number) number {
  Baz'Foo(x)
}
```

### Module names

#### Modules in the same package

Modules in the same package are referenced by their paths relative to their package root directories.

For example, a module of a file `<package directory>/Foo/Bar.pen` is imported as below.

```
import 'Foo'Bar
```

#### Modules in other packages

Modules in other packages are referenced by their package names and module paths.

For example, a module of a file `<package directory>/Bar/Baz.pen` in a package `Foo` is imported as below.

```
import Foo'Bar'Baz
```

### Custom prefixes

> This feature is work in progress.

Imported modules can have prefixes different from their names.

```
import Bar Foo'Bar'Baz
```
