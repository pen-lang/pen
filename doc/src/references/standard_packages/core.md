# Core

## Install

```json
{
  "dependencies": {
    "Core": "file://pen-root/lib/core"
  }
}
```

## `Number` module

```pen
import Core'Number
```

### Functions

#### `String`

It converts a number to its string representation.

```pen
\(number) string
```

## `String` module

```pen
import Core'String
```

### Functions

#### `Join`

It joins a list of strings into a string.

```pen
\([string]) string
```

#### `Slice`

It slices a string with start and end indexes.

```pen
\(string, number, number) string
```
