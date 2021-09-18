# Modules

Modules are sets of functions and types. Using modules, you can split large programs into smaller chunks to make them comprehensive and reusable.

Each source file suffixed with a `.pen` file extension composes a module. Modules are exported to and imported from other modules.

## Exporting functions and types from modules

You can name functions and types in an upper camel case for them to be accessible from other modules by [import statements](#importing-functions-and-types-from-modules).

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

In order to import functions and types from other modules, place [import statements](/references/language/syntax.md#import-statement) at the top of the current module.

The first components in the statements are names of external packages you declare in [package configuration files][package-configuration] (`Foo`.) They are omitted if the imported modules are in the same packages. The rest of the components are directory names where the modules exist (`Bar`) and the modules' filenames without their file extensions (`Baz` for `Baz.pen`.)

```pen
import Foo'Bar'Baz
```

Then, you can access exported members of the modules with their prefixes.

```pen
type foo = Baz'Type

bar = \(x number) number {
  Baz'Function(x)
}
```

### Module names

#### Modules in the same package

Modules in the same package are referenced by their paths relative to a root directory of the package.

For example, a module of a file at `<package directory>/Foo/Bar.pen` is imported as below.

```pen
import 'Foo'Bar
```

#### Modules in other packages

Modules in other packages are referenced by their package names defined in [package configuration files][package-configuration] and module paths.

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

### Unqualified import

You can import functions and types without prefixes by putting their names between `{` and `}` in [import statements](/references/language/syntax.md#import-statement). This is especially useful when module names and imported entities have the same names like `import 'MyType { MyType }`.

```pen
import Foo'Bar { Foo, Bar }

type Baz {
  foo Foo
}

Blah = \() number {
  Bar()
}
```

[package-configuration]: packages.md#package-configuration
