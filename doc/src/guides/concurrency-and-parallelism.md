# Concurrency and parallelism

Concurrent programs use CPU time efficiently without blocking on I/O or synchronization. And parallel programs leverage multi-core CPUs to compute something in parallel for speed.

Pen provides simple syntax for concurrent and/or parallel programming. The `go` expression of the `go` keyword followed by a function with no argument runs the function in a separate execution context.

```pen
f = go \() number {
  computeExpensive(x, y, z)
}
```

The `go` expression returns a function of the type same as the given expression. The function returns a resulting value of the function execution.

The `go` expression may or may not block the current execution context depending on its implementation. For example, the built-in `Os` package runs the given function in parallel if multiple cores are available in CPUs.
