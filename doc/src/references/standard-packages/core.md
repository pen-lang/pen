# `Core` package

## `Core'Bit` module

A module for bitwise operations

Most functions defined in this module take arguments of 64-bit integers.
They can be converted from and into an integer represented in IEEE-754 of the

### Types

No types are defined.

### Functions

#### `And`

Calculate bitwise "and" given two 64-bit integers.

```pen
\(x number, y number) number
```

#### `Or`

Calculate bitwise "or" given two 64-bit integers.

```pen
\(x number, y number) number
```

#### `Xor`

Calculate bitwise exclusive-"or" given two 64-bit integers.

```pen
\(x number, y number) number
```

#### `Not`

Calculate bitwise "not" given two 64-bit integers.

```pen
\(x number) number
```

#### `LeftShift`

Calculate unsigned left shift given a 64-bit integer.

```pen
\(x number, n number) number
```

#### `RightShift`

Calculate unsigned right shift given a 64-bit integer.

```pen
\(x number, n number) number
```

#### `ToInteger64`

Convert an integer in IEEE-754 to a 64-bit integer.

```pen
\(x number) number
```

#### `FromInteger64`

Convert a 64-bit integer to an integer in IEEE-754.

```pen
\(x number) number
```

## `Core'List` module

### Types

No types are defined.

### Functions

#### `Length`

```pen
\(xs [any]) number
```

#### `First`

```pen
\(xs [any], fallback any) any
```

#### `Last`

```pen
\(xs [any], fallback any) any
```

#### `ToNumbers`

```pen
\(xs [any]) [number]
```

#### `ToStrings`

```pen
\(xs [any]) [string]
```

## `Core'Map` module

### Types

#### `Map`

```pen
type Map {
  ...
}
```

### Functions

#### `New`

```pen
\(equalKeys \(any, any) boolean, hashKey \(any) number) Map
```

#### `Size`

```pen
\(map Map) number
```

#### `Get`

```pen
\(map Map, key any, default any) any
```

#### `Set`

```pen
\(map Map, key any, value any) Map
```

#### `Delete`

```pen
\(map Map, key any) Map
```

#### `Merge`

```pen
\(x Map, y Map) Map
```

#### `Keys`

```pen
\(map Map) [any]
```

#### `Values`

```pen
\(m Map) [any]
```

## `Core'Map'NumberMap` module

### Types

#### `Map`

```pen
type Map {
  ...
}
```

### Functions

#### `New`

```pen
\() Map
```

#### `Get`

```pen
\(map Map, key number, default any) any
```

#### `Set`

```pen
\(map Map, key number, value any) Map
```

#### `Delete`

```pen
\(map Map, key number) Map
```

#### `Size`

```pen
\(map Map) number
```

#### `Merge`

```pen
\(x Map, y Map) Map
```

#### `Keys`

```pen
\(map Map) [number]
```

#### `Values`

```pen
\(map Map) [any]
```

## `Core'Map'StringMap` module

### Types

#### `Map`

```pen
type Map {
  ...
}
```

### Functions

#### `New`

```pen
\() Map
```

#### `Get`

```pen
\(map Map, key string, default any) any
```

#### `Set`

```pen
\(map Map, key string, value any) Map
```

#### `Delete`

```pen
\(map Map, key string) Map
```

#### `Size`

```pen
\(map Map) number
```

#### `Merge`

```pen
\(x Map, y Map) Map
```

#### `Keys`

```pen
\(map Map) [string]
```

#### `Values`

```pen
\(map Map) [any]
```

## `Core'Number` module

### Types

No types are defined.

### Functions

#### `String`

```pen
\(x number) string
```

#### `Sum`

```pen
\(ns [number]) number
```

#### `Remainder`

```pen
\(x number, y number) number
```

#### `Power`

```pen
\(x number, y number) number
```

#### `SquareRoot`

```pen
\(x number) number
```

#### `Sequence`

```pen
\(n number) [number]
```

#### `Range`

```pen
\(min number, max number) [number]
```

#### `IsNan`

```pen
\(x number) boolean
```

#### `Ceil`

```pen
\(x number) number
```

#### `Floor`

```pen
\(x number) number
```

## `Core'String` module

### Types

No types are defined.

### Functions

#### `Join`

```pen
\(ss [string], sep string) string
```

#### `Slice`

```pen
\(s string, start number, end number) string
```
