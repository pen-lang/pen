# Concurrency and parallelism

Concurrent programs use CPU time efficiently without blocking on I/O or synchronization. And parallel programs leverage multi-core CPUs to compute something in parallel for speed.

Pen provides simple syntax for concurrent and parallel programming. Its `go` expression runs a given function concurrently, and possibly in parallel. The expression is composed of the `go` keyword followed by a function with no argument.

```pen
f = go \() number {
  computeExpensive(x, y, z)
}
```

The `go` expression returns a function of the type same as the given expression (a.k.a. futures or promises in other languages.) The function returns a resulting value of the function execution.

The `go` expression may or may not run a given function immediately depending on its implementation. For example, the standard `Os` system package runs the given function in parallel if multiple CPU cores are available.
