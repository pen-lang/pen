# Built-ins

Built-in types and functions are ones implicitly defined in every module.

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
\(info any) error
```

### `source`

It extracts source information from an error.

```pen
\(e error) any
```
