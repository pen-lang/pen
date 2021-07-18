---
title: Built-ins
---

# Built-ins

Built-in types and functions that are accessible from any modules

## Types

### `error`

`error` type is a special record type used for error handling.

```
type error {
  ...
}
```

## Functions

### `not`

`not` function flips a condition.

```
not : Boolean -> Boolean
```

### `error`

`error` function creates an error with its source information.

```
error : Any -> Error
```

### `source`

`source` function extracts source information from an error.

```
source : Error -> Any
```
