# Packages

Packages are sets of [modules](/references/language/modules.md). Like modules, packages can import other packages specifying them in [their configurations](#package-configuration).

## What composes a package?

The following entities compose packages.

- Standard packages bundled in [installation](/guides/install.md) of the language
- Remote repositories managed by version control systems (VCS)
  - Currently, only [Git](https://git-scm.com/) is supported as a VCS.
- Directories with [package configuration files](#package-configuration) on file systems

## Package types

There are 3 package types: application, library, and system. Those types are specified in [package configuration files](#package-configuration).

- Application packages build applications, often, of executable files.
- Library packages are imported and used by other packages.
- System packages are similar to library packages but provide system interfaces to application packages.

Packages are considered as application packages if they have `main.pen` files at their top directories. Note that every application package needs to have at least [one system package](/advanced-features/system-injection.md#system-packages) in its [package configuration file](#package-configuration).

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
