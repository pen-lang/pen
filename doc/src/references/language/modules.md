# Modules

Modules are sets of functions and types. You can split programs into reasonable sizes of modules to make them comprehensive and reusable.

Each source file suffixed with a `.pen` file extension composes a module. Modules are exported to and imported from other modules.

## Exporting functions and types from modules

You can name functions and types in an upper camel case for them to be accessible from other modules using [import statements](#importing-functions-and-types-from-modules).

```pen
type Foo {
  ...
}

type Bar = ...

Foo = \() number {
  ...
}
```

## Importing functions and types from modules

In order to import functions and types from a module, first, place [an `import` statement](/references/language/syntax.md#import-statement) with the name of the module you want to import at the top of the current module.

The first component of a module name in the statement is a name of a package you declare in [a `pen.json` file][package-configuration] (`Foo`.) It is omitted if the imported module is in the same package as the current one. The rest of the components are directory names where a module exists (`Bar`) and the basename of the module filename without its file extension (`Baz` for `Baz.pen`.)

```pen
import Foo'Bar'Baz
```

Then, you can access exported members of the module with its prefix.

```pen
type Foo = Baz'Type

bar = \(x number) number {
  Baz'Function(x)
}
```

### Module names

#### Modules in the same package

Modules in the same package are referenced by their paths relative to the root directory of the package.

For example, a module of a file at `<package directory>/Foo/Bar.pen` is imported as below.

```pen
import 'Foo'Bar
```

#### Modules in other packages

Modules in other packages are referenced by their package names defined in [`pen.json` files][package-configuration] and module paths.

For example, a module of a file at `<package directory>/Bar/Baz.pen` in a package `Foo` is imported as below.

```pen
import Foo'Bar'Baz
```

### Custom prefixes

Imported modules can have custom prefixes given different names after the `as` keywords.

```pen
import Foo'Bar'Baz as Blah
```

[package-configuration]: packages.md#package-configuration
