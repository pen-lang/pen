# Foreign Function Interface (FFI)

Using FFI, you can import or export functions in foreign languages, such as [Rust](https://www.rust-lang.org/) and C.

## Importing functions in foreign languages

You can import functions in foreign languages using [foreign import statements](/references/language/syntax.md#foreign-import-statement). The statements specify the foreign functions' calling convention, names and types.

You might specify calling conventions of foreign functions in a format of string literals after `import foreign` keywords optionally. Currently, only the C calling convention is supported as `"c"`. If the options are omitted, the functions are imported with the native calling convention of Pen.

```pen
import foreign "c" foo \(number, number) number
```

> Caveat: You can import foreign functions that might cause side effects **only in system packages**. See also [Writing system packages](writing-system-packages.md).

## Exporting functions to foreign languages

You can export functions to foreign languages using [foreign function definitions](/references/language/syntax.md#foreign-function-definition), which have `foreign` keywords in front of normal function definitions.

You might specify calling conventions of exported foreign functions optionally after `foreign` keywords as well as [imported foreign functions](#importing-functions-in-foreign-languages).

```pen
foreign "c" foo = \(x number, y number) number {
  ...
}
```

## Building libraries of foreign languages

During builds of your packages, you might want to build libraries of foreign languages so that you can use their functions. If that's your case, you can set up `pen-ffi` scripts in your packages. The script files run on every build and output absolute paths to `.a` archive files of your libraries in foreign languages built by the scripts. The script files may or may not have file extensions.

The `pen-ffi` scripts should accept the following command line arguments.

| Argument      | Required | Description                    |
| ------------- | -------- | ------------------------------ |
| `-t <target>` | No       | Custom target triple of builds |

One of examples in practice is [a `pen-ffi.sh` file in the `Core` library](https://github.com/pen-lang/pen/blob/main/packages/core/pen-ffi.sh).

## Native calling convention in Pen

> TBD
