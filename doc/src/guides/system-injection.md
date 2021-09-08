# System injection

System injection is one of the biggest features of the language where you can define your own system APIs and build applications in your own formats.

## System packages

In [their package configurations](/references/language/packages.md#package-configuration), application packages need to specify system packages that are special kinds of library packages and define system APIs for the programs to interact with the world outside.

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
