---
title: Packages
---

# Packages

## Overview

- Each repository of a version control system (VCS) or directory with a package configuration file on a file system composes a package.
  - Currently, only [Git](https://git-scm.com/) is supported as a VCS.
- A package contains multiple modules.
- Packages are either application or library packages.
  - Packages are considered to be of applications if they have `Main.pen` files at their top directories. Otherwise, they are library ones.
- Modules in library packages can be imported from other packages.
- To import modules in other packages, see [Modules](../modules).

## Package configuration

- Each package has its configuration file named `pen.json` at its top directory.
- The configuration file is in JSON.
- It has a single field named `dependencies` specifying package names and URLs to their locations.
  - Every application package needs to have a system package entry named `System`.

### Examples

#### Application

```json
{
  "dependencies": {
    "System": "file://pen-root/lib/os",
    "Core": "file://pen-root/lib/core",
    "MyLibrary": "git://github.com/john-doe/super-hello-world"
  }
}
```

#### Library

```json
{
  "dependencies": {
    "Core": "pen://pen-root/lib/core",
    "MyLibrary": "git://github.com/john-doe/super-hello-world"
  }
}
```
