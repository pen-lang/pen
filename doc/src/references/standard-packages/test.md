# Test

This package provides utility functions and types for testing.

## Install

```json
{
  "dependencies": {
    "Test": "pen:///lib/test"
  }
}
```

## `Assert` module

```pen
import Test'Assert
```

### Functions

#### `True`

It returns `none` if a given condition is `true`, or returns an error otherwise.

```pen
\(boolean) none | error
```

#### `Fail`

It always fails with an error.

```pen
\() error
```

#### `EqualNumbers`

It returns `none` if given numbers are equal, or returns an error otherwise.

```pen
\(number, number) none | error
```

#### `EqualStrings`

It returns `none` if given strings are equal, or returns an error otherwise.

```pen
\(string, string) none | error
```
