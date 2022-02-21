# Packages

Packages are sets of [modules](/references/language/modules.md). Like modules, packages can import other packages specifying them in [their configurations](#package-configuration).

## What composes a package?

The following entities compose packages.

- Standard packages bundled in [installation](/guides/install.md) of the language
- Remote repositories managed by version control systems (VCS)
  - Currently, only [Git](https://git-scm.com/) is supported as a VCS.
- Directories with [package configuration files](#package-configuration) on file systems

During builds of packages, the language's build system automatically download and initialize their dependency packages based on their URLs.

## Package types

There are 3 package types: application, library, and system. Those types are specified in [package configuration files](#package-configuration).

- Application packages build applications, often, of executable files.
- Library packages are imported and used by other packages.
- System packages are similar to library packages but provide system interfaces to application packages.

### Application packages

Application packages must have `main.pen` module files at their top directories. Those main modules have a `main` function that receives an argument of a `context` type and returns a `none` type. The `context` type is a record type containing context values of system packages with their field names of package names. For example, given system packages named `Http` and `Os`, a main function looks like the following.

```pen
main = \(ctx context) none {
  s = fetch(ctx.Http, "https://pen-lang.org/")
  print(ctx.Os, s)

  none
}
```

Every application package must specify one and only one [system package](/advanced-features/writing-system-packages.md#system-packages) that links applications (e.g. the `Os` standard system package) in its [package configuration file](#package-configuration). Otherwise, their builds fail. However, application packages can specify system packages that do no link applications (e.g. the `Http` system package in the example above) as many as possible.

### Library packages

> WIP

### System packages

> WIP

## Package configuration

Each package has its configuration file named `pen.json` in a [JSON](https://www.json.org/json-en.html) format at its top directory. The JSON file has a field named `type` specifying its type and a field named `dependencies` specifying names and URLs of external packages.

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
