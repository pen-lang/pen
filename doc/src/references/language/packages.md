# Packages

Packages are sets of [modules](/references/language/modules.md). Like modules, packages can import other packages specifying them in [their configurations](#package-configuration).

## What composes a package?

The following entities compose packages.

- Standard packages bundled in [installation](/guides/install.md) of the language
- Remote repositories managed by version control systems (VCS)
  - Currently, only [Git](https://git-scm.com/) is supported as a VCS.
- Directories with [package configuration files](#package-configuration) on file systems

## Kinds of packages

There are 3 kinds of packages: application, library, and system. Application packages build applications usually of executable files. Library packages are imported and used by other packages of any kinds. System packages are similar to library packages but provide system interfaces to application packages.

Packages are considered as application packages if they have `main.pen` files at their top directories. Packages are considered as system packages if their [package configuration files](#package-configuration) have a `"system"` field set to `true`. Otherwise, they are library packages. Note that every application package needs to have at least [one system package](/advanced-features/system-injection.md#system-packages) in its [package configuration file](#package-configuration).

## Package configuration

Each package has its configuration file named `pen.json` in a [JSON](https://www.json.org/json-en.html) format at its top directory. The JSON file has a single field named `dependencies` specifying names and URLs of packages to import.

Package URLs have different protocol schemes depending on where they are located.

- Standard packages: `pen`
- Git repositories: `git`
- Directories on file systems: none

### Examples

#### Application

```json
{
  "dependencies": {
    "Os": "pen:///os",
    "Core": "pen:///core",
    "MyLibrary": "git://github.com/john-doe/super-hello-world"
  }
}
```

#### Library

```json
{
  "dependencies": {
    "Core": "pen:///core",
    "MyLibrary": "git://github.com/john-doe/super-hello-world"
  }
}
```
