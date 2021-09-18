# Foreign Function Interface (FFI)

Using FFI, you can import or export functions in foreign languages, such as [Rust](https://www.rust-lang.org/) and C.

## Importing functions in foreign languages

You can import functions in foreign languages using [foreign import statements](/references/language/syntax.md#foreign-import-statement). The statements specify the foreign functions' calling convention, names and types. After imported, the functions are available under the names.

```pen
import foreign "c" foo \(number, number) number
```

> Caveat: You can import foreign functions that might make side effects **only in [system packages](system-injection.md#system-packages)**. See also [System injection](system-injection.md).

## Exporting functions to foreign languages

You can export functions to foreign languages using [foreign function definitions](/references/language/syntax.md#foreign-function-definition) which have `foreign` keywords in front of normal function definitions.

```pen
foreign foo = \(x number, y number) number {
  ...
}
```

## Building custom libraries in foreign languages in packages

You might want to build libraries in foreign languages so that you can use their functions in your packages. If that's your case, you can set up `pen-ffi` scripts in your packages. The script files run on every build and output absolute paths to `.a` archive files of your libraries in foreign languages built by the scripts. The script files may or may not have file extensions.

The `pen-ffi` scripts should accept the following command line arguments.

| Argument      | Required | Description                    |
| ------------- | -------- | ------------------------------ |
| `-t <target>` | No       | Custom target triple of builds |

One of examples in practice is [a `pen-ffi.sh` file in the core library](https://github.com/pen-lang/pen/blob/main/lib/core/pen-ffi.sh).
