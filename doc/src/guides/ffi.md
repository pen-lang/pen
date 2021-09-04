# Foreign Function Interface (FFI)

Using FFI, you can import or export functions in foreign languages.

## Importing functions in foreign languages

You can import functions in foreign languages using [foreign import statements](/references/language/syntax.md#foreign-import-statement) which specify the functions' calling convention, names and types. Then, the functions are available under the names.

```pen
import foreign "c" foo \(number, number) number
```

## Exporting functions to foreign languages

You can export functions to foreign languages using [foreign function definitions](/references/language/syntax.md#foreign-function-definition) which have `export foreign` keywords in front of normal function definitions.

```pen
export foreign foo = \(x number, y number) number {
  ...
}
```
