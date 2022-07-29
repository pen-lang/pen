# Built-ins

Built-in types and functions are ones implicitly defined in every module.

## Types

See [Types](types.md).

## Functions

### `size`

It calculates a size of a list or a map. It is generic and you can apply it to any list and map types.

Its time complexity is O(n) for lists and O(1) for maps.

```pen
\(list [a]) number
\(map {a: b}) number
```

### `error`

It creates an error with its source information.

```pen
\(s any) error
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

### `go`

It executes a function concurrently. Its return value is a future represented as a function that returns a result of the executed function.

```pen
\(\() a) \() a
```

### `race`

It merges multiple lists into one evaluating elements in each list concurrently. This function corresponds to [the fan-in concurrency pattern](https://go.dev/blog/pipelines#fan-out-fan-in) in other languages where we merge results of concurrent computation in multiple concurrent queues.

```pen
\([[a]]) [a]
```
