# Concurrency and parallelism

Concurrent programs use CPU time efficiently without blocking on I/O or synchronization. And parallel programs leverage multi-core CPUs to compute something in parallel for speed.

Pen provides some bulit-in functions for concurrent and parallel programming. Its `go` built-in function runs a given function concurrently, and possibly in parallel.

```pen
f = go(\() number {
  computeExpensive(x, y, z)
})
```

The `go` function returns a function of the same type as the given argument (a.k.a. futures or promises in other languages.) The returned function returns a resulting value of the function execution.

The `go` function may or may not run a given function immediately depending on its implementation. For example, the standard `Os` system package runs the given function in parallel if multiple CPU cores are available.
