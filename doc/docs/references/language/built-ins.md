# Built-ins

Built-in types and functions are ones implicitly defined in every module.

## Types

See [Types](types.md).

## Functions

### `size`

It calculates a size of a list or map. It is generic and you can apply it to any list and map types.

Its time complexity is O(n) for lists and O(1) for maps.

```pen
\(list [a]) number
\(map {k: v}) number
```

### `delete`

It deletes a key from a map.

```pen
\(map {k: v}, key k) {k: v}
```

### `keys`

It gets keys in a map.

```pen
\(map {k: v}) [k]
```

### `values`

It gets values in a map.

```pen
\(map {k: v}) [v]
```

### `error`

It creates an error with its source information.

```pen
\(x any) error
```

### `source`

It extracts source information from an error.

```pen
\(e error) any
```

### `debug`

It pretty-prints a given value.

Note that behavior of this function can change among system packages. **You may not even see any messages with system packages whose systems do not have any consoles.**

```pen
\(x any) none
```

### `go`

It executes a function concurrently. Its return value is a future represented as a function that returns a result of the executed function.

```pen
\(\() a) \() a
```

### `race`

It merges multiple lists into one by evaluating elements in the lists concurrently. Elements in each list are evaluated sequentially in order.

This function corresponds to [the fan-in concurrency pattern](https://go.dev/blog/pipelines#fan-out-fan-in) in other languages where results of concurrent computation in multiple queues are merged into a queue.

```pen
\([[a]]) [a]
```
