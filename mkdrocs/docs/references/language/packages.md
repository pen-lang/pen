# Packages

Packages are sets of [modules](/references/language/modules.md). Like modules, packages can import other packages specifying them in [their configurations](#package-configuration).

## What composes a package?

The following entities compose packages.

- Standard packages bundled in [installation](/introduction/install.md) of the language
- Remote repositories managed by version control systems (VCS)
  - Currently, Pen supports only [Git](https://git-scm.com/) as a VCS.
- Directories with [package configuration files](#package-configuration) on file systems

During builds of packages, Pen's build system automatically download and initialize their dependency packages based on their URLs.

## Package types

There are 3 package types: application, library, and system. Those types are specified in [package configuration files](#package-configuration).

### Application packages

Application packages build applications often as executable files. Every application package must have a `main.pen` module file at its top directory. The main module has a `main` function that receives an argument of a `context` type and returns a `none` type. The `context` type is a record type containing context values of system packages with their field names of package names. For example, given system packages named `Http` and `Os`, a main function looks like the following.

```pen
main = \(ctx context) none {
  s = fetch(ctx.Http, "https://pen-lang.org/")
  print(ctx.Os, s)

  none
}
```

Every application package must specify one and only one [system package](#system-packages) that links applications (e.g. the `Os` standard system package) in its [package configuration file](#package-configuration). However, application packages can specify system packages that do not link applications (e.g. the `Http` system package in the example above) as many as possible.

### Library packages

Library packages contain functions and types that have _no_ side effects. They are imported and used by other packages.

### System packages

System packages contain functions and types that have side effects to provide system interfaces to application packages. The language currently provides the two standard system packages of `Os` and `OsSync`.

Although they can be imported by library packages as well as application packages, then they are expected not to cause any side effects.

If you want to write your own system packages, see [Writing system packages](/advanced-features/writing-system-packages.md).

## Package configuration

Each package has its configuration file named `pen.json` in a [JSON](https://www.json.org/json-en.html) format at its top directory. The JSON file has the following fields.

| Name           | Required | Description                                                 |
| -------------- | -------- | ----------------------------------------------------------- |
| `type`         | Yes      | Package type (either `application`, `library`, or `system`) |
| `dependencies` | Yes      | Map of package names to their URLs                          |

Package URLs have different protocol schemes depending on where they are located.

- Standard packages: `pen`
- Git repositories: `git`
- Directories on file systems: none

### Examples

#### Application

```json
{
  "type": "application",
  "dependencies": {
    "Os": "pen:///os",
    "Core": "pen:///core",
    "Foo": "git://github.com/foo/foo",
    "Bar": "../bar"
  }
}
```

#### Library

```json
{
  "type": "library",
  "dependencies": {
    "Core": "pen:///core",
    "Foo": "git://github.com/foo/foo",
    "Bar": "../bar"
  }
}
```

#### System

```json
{
  "type": "system",
  "dependencies": {
    "Core": "pen:///core",
    "Foo": "git://github.com/foo/foo",
    "Bar": "../bar"
  }
}
```
