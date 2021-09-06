# Packages

Packages are sets of modules. They are built by the `pen build` command as applications or libraries depending on their configurations.

## What composes a package?

The following entities can compose packages.

- A repository of a version control system (VCS)
  - Currently, only [Git](https://git-scm.com/) is supported as a VCS.
- A directory with [a package configuration file](#package-configuration) on a file system composes a package.
- The standard packages that come with the `pen` command
  - They have a special protocol scheme of `pen` if imported in package configuration files.

## Kinds of packages

There are two kinds of packages: applications and libraries.

Applicaiton packages build applications that are often executable files on host platforms. Library packages are meant to be used by other packages of both applications and libraries by being imported there.

Packages are considered to be of applications if they have `Main.pen` files at their top directories. Otherwise, they are library ones.

Note that every application package needs to have a system package entry named `System` defined in their package configuration files.

## Package configuration

Each package has its configuration file named `pen.json` in a [JSON](https://www.json.org/json-en.html) format at its top directory. The JSON files has a single field named `dependencies` specifying package names and URLs to their locations.

The package URLs have different formats.

### Examples

#### Application

```json
{
  "dependencies": {
    "System": "pen:///lib/os",
    "Core": "pen:///lib/core",
    "MyLibrary": "git://github.com/john-doe/super-hello-world"
  }
}
```

#### Library

```json
{
  "dependencies": {
    "Core": "pen:///lib/core",
    "MyLibrary": "git://github.com/john-doe/super-hello-world"
  }
}
```
