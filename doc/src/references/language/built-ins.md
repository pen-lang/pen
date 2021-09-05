# Built-ins

Built-in types and functions are accessible from any modules.

## Types

### `error`

It is a special record type used for error handling. See also [Error handling](/references/language/syntax.md#error-handling).

```pen
type error {
  ...
}
```

## Functions

### `error`

It creates an error with its source information.

```pen
error \(any) error
```

### `source`

It extracts source information from an error.

```pen
source \(error) any
```
