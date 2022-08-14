# Concurrency and parallelism

Concurrent programs use CPU time efficiently without being blocked on I/O or data synchronization. Parallel programs leverage multi-core CPUs to compute things in parallel faster than sequential programs.

This page describes how to write concurrent and/or parallel programs in Pen.

## Built-ins

Pen provides several built-in functions for concurrent and parallel programming.

### `go` function

[The `go` built-in function](/references/language/built-ins.html#go) runs a given function concurrently, and possibly in parallel.

```pen
future = go(\() number {
  computeExpensive(x, y, z)
})
```

The `go` function returns a function of the same type as the given argument. The returned function returns a resulting value of the function execution. In other languages, such functions returning values computed concurrently when they are ready are also known as [futures](https://doc.rust-lang.org/std/future/trait.Future.html) or [promises](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise).

The `go` function may or may not run a given function immediately depending on its implementation. For example, [the standard `Os` system package](/references/standard-packages/os.md) runs the given function in parallel if multiple CPU cores are available.

### `race` function

[The `race` built-in function](/references/language/built-ins.html#race) takes multiple lists and merge them into one by evaluating elements in each list concurrently and possibly in parallel. The resulting list contains the elements in the original lists in order of their finished times of computation. Remember that elements in lists are evaluated lazily.

```pen
zs = race([[number] xs, ys])
```

This functionality is similar to concurrent queues in other imperative languages, such as [channels](https://go.dev/tour/concurrency/2) in [Go][go]. Input lists to the `race` function correspond to producers of elements into the queue, and a consumer of the queue is codes that use elements in the output list.

## Patterns

### Task parallelism

The `go` function can run different codes concurrently. For example, the following code runs the functions, `computeA` and `computeB` concurrently. Runtimes of applications might execute those functions even in parallel if their system packages allow that.

```pen
compute = \(x number, y number) number {
  z = go(\() number { computeA(x) })
  v = computeB(y)

  v + z
}
```

### Data parallelism

To run the same computation against many values of the same type, you can use recursion and the `go` function.

```pen
computeMany = \(xs [number]) [number] {
  if [x, ...xs] = xs {
    y = go(\() number { foo(x()) })
    ys = computeMany(xs)

    [number y(), ...ys]
  } else {
    [number]
  }
}
```

The example above computes things in order of elements in the original list. However, you might want to see output values of concurrent computation in order of their finished times. By doing that, you can start using the output values as fast as possible without waiting for all computation to be completed. In this case, you can use the `race` function to reorder elements in the output list by their finished times.

```pen
compute = \(xs [number]) [number] {
  race([[number] [number x()] for x in computeMany(xs)])
}
```

If you want to evaluate elements in multiple lists concurrently, you can simply pass the lists as an argument to the `race` function. Note that elements in the same lists are not evaluated concurrently although elements in different lists are evaluated concurrently.

```pen
compute = \(xs [number], ys [number]) [number] {
  race([[number] computeMany(xs), computeMany(ys)])
}
```

[go]: https://go.dev
