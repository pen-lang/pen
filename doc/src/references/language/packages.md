# Packages

Packages are sets of [modules](/references/language/modules.md). Packages are built as either applications or libraries. Like modules, packages can use other packages specified in [their configurations](#package-configuration).

## What composes a package?

The following entities compose packages.

- Standard packages that come with [installation](/guides/install.md) of the language
- Repositories of version control systems (VCS)
  - Currently, only [Git](https://git-scm.com/) is supported as a VCS.
- Directories with [package configuration files](#package-configuration) on file systems

## Kinds of packages

There are two kinds of packages: applications and libraries. Application packages build applications which are often executable files on host platforms. Library packages are meant to be imported and used by other packages which can be of both applications and libraries.

Packages are considered to be of applications if they have `main.pen` files at their top directories. Otherwise, they are of libraries. Note that every application package needs to specify [a system package](/advanced-features/system-injection.md#system-packages) with a key of `System` in its [package configuration file](#package-configuration).

## Package configuration

Each package has its configuration file named `pen.json` in a [JSON](https://www.json.org/json-en.html) format at its top directory. The JSON file has a single field named `dependencies` specifying external packages' names and URLs.

Package URLs have different protocol schemes depending on where they are located.

- Standard packages: `pen`
- Git repositories: `git`
- Directories on file systems: `file`

### Examples

#### Application

```json
{
  "dependencies": {
    "System": "pen:///os",
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
