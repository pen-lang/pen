---
title: Packages
---

# Packages

- Each repository of some version control system (VCS) composes a package.
  - Currently, only [Git](https://git-scm.com/) is supported.
- Packages contain modules.
- Packages are either application or library packages.
- Modules in library packages can be imported by other packages.

## Package names

Packages are referenced by host names and paths in their VCS's URLs. For example, a package of a Git repository at a URL of `https://github.com/foo/bar` is referenced as `github.com/foo/bar`.

To import modules in other packages, see [Modules](modules.md).

## Package configuration

- Each package has its configuration file named `pen.json` at its root directory.

### Configuration file format

- Packages are considered to be applications if they have `application` fields.

| Field                                 | Required | Description                                                     |
| ------------------------------------- | -------- | --------------------------------------------------------------- |
| `application`                         | No       | Application configuration                                       |
| `application.name`                    | Yes      | Application name                                                |
| `application.system`                  | Yes      | System package configuration                                    |
| `application.system.name`             | Yes      | System package name. See the `dependencies` field.              |
| `application.system.version`          | Yes      | System package version. See the `dependencies` field.           |
| `dependencies`                        | Yes      | Dependent packages as a map from names to their configurations. |
| `dependencies.<package name>.version` | Yes      | A version of a package. For Git, they are branch names.         |

### Examples

#### Application

```json
{
  "application": {
    "name": "foo",
    "system": {
      "name": "github.com/pen-lang/os",
      "version": "main"
    }
  },
  "dependencies": {
    "github.com/foo/bar": { "version": "main" }
  }
}
```

#### Library

```json
{
  "dependencies": {
    "github.com/foo/bar": { "version": "main" }
  }
}
```
