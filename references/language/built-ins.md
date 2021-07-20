# Built-ins

Built-in types and functions that are accessible from any modules

## Types

### `error`

It is a special record type used for error handling.

```
type error {
  ...
}
```

## Functions

### `error`

It creates an error with its source information.

```
error \(any) error
```

### `source`

It extracts source information from an error.

```
source \(error) any
```
