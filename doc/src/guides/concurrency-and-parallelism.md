# Concurrency and parallelism

Concurrent programs use CPU time efficiently without being blocked on I/O or data synchronization. And parallel programs leverage multi-core CPUs to compute things in parallel faster than sequential programs.

This page describes how to write concurrent and/or parallel programs in Pen.

## Built-ins

### `go` function

Pen provides some built-in functions for concurrent and parallel programming. [The `go` built-in function](/references/language/built-ins.html#go) runs a given function concurrently, and possibly in parallel.

```pen
f = go(\() number {
  computeExpensive(x, y, z)
})
```

The `go` function returns a function of the same type as the given argument (a.k.a. futures or promises in other languages.) The returned function returns a resulting value of the function execution.

The `go` function may or may not run a given function immediately depending on its implementation. For example, the standard `Os` system package runs the given function in parallel if multiple CPU cores are available.

### `race` function

[The `race` built-in function](/references/language/built-ins.html#race) takes multiple lists and merge them into one by evaluating elements in each list concurrently, and possibly in parallel. The resulting list contains the elements in the original lists in order of finished times of computation.

```pen
zs = race([[number] xs, ys])
```

This is similar to concurrent queues in other imperative languages, such as channels in [Go][go]. Input lists to the `race` function corresponds to producers of elements in the queue, and a consumer of the queue is codes that use the output list.

## Patterns

### Task parallelism

You can use the `go` function to run different codes concurrently. For examlpe, the following code runs the functions, `calculateA` and `calculateB` concurrently. Runtimes of applications might execute those functions even in parallel if runtimes of their system packages allow that,

```pen
calculate = \(x number, y number) number {
  z = go(\() number { calculateA(x) })
  v = calculateB(y)

  v + z
}
```

### Data parallelism

To run the same computation against many pieces of the same kind of data, you can use recursion and the `go` function.

```pen
calculate = \(x [number]) [number] {
  z = go(\() number { foo(x) })
  v = bar(y)

  v + z
}
```

[go]: https://go.dev
