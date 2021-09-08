# System injection

Using system injection, you can define your own system APIs and build applications in file formats of your choice. Those system APIs and application linking logic are bundled into [system packages](#system-packages).

## System packages

In [their package configurations](/references/language/packages.md#package-configuration), [application packages](/references/language/packages.md#kinds-of-packages) need to specify system packages that are special kinds of library packages and define system APIs for the programs to interact with the world outside.

System packages do the following three things.

### Define main function types

A system package has a module named `MainFunction` where a `MainFunction` function type is defined at the top level.

### Providing system APIs

> WIP

### Linking application files

> WIP

## `pen-link` scripts

Each system package has a `pen-link` script file at the its directory. The executable file is run with object files specified by command line arguments to build an application file on every build. The script files may or may not have file extensions.

The scripts should accept the following command line arguments. Outputs of the scripts are simply discarded unless any error occurs during runs.

| Argument           | Required | Description                         |
| ------------------ | -------- | ----------------------------------- |
| `-t <target>`      | No       | Target triple of builds             |
| `-o <application>` | Yes      | Path of a application file to build |
| `<archive>...`     | Yes      | Paths of archive files to link      |

## Examples

[The OS standard package](https://github.com/pen-lang/pen/tree/main/lib/os) is one of examples of system packages.
