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

It fails and returns an error if a given condition is `false`, and returns `none` otherwise.

```pen
\(boolean) none | error
```
