# Built-ins

Built-in types and functions are ones implicitly defined in every module.

## Types

### `error`

It is a special type used for error handling. See also [Error handling](/references/language/syntax.md#error-handling).

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

### `debug`

It prints a debug message given as an argument if a `PEN_DEBUG` environment variable is set.

Note that behavior of this function can change among system packages. **You may not even see any messages with system packages whose systems do not have any consoles.**

```pen
\(message string) none
```
