# Foreign Function Interface (FFI)

Using FFI, you can import or export functions in foreign languages.

## Importing functions in foreign languages

You can import a function in a foreign language using [foreign import statements](/references/language/syntax.md#foreign-import-statement).
You need to specify the function's calling convention, name and type.

```pen
import foreign "c" foo \(number, number) number
```

## Exporting functions for foreign languages

```pen
export foreign foo
```
