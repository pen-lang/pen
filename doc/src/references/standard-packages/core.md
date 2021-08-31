# Core

## Install

```json
{
  "dependencies": {
    "Core": "pen:///lib/core"
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

#### `Sum`

It calculates a sum of numbers.

```pen
\([number]) number
```

## `List` module

```pen
import Core'List
```

### Functions

#### `Length`

It returns a length of a list.

```pen
\([any]) number
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
