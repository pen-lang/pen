# Memory leak

## Background

_Given_ a file named "pen.json" with:

```json
{
  "dependencies": {
    "Core": "file://pen-root/lib/core",
    "System": "file://pen-root/lib/os"
  }
}
```

## Run an infinite loop

_Given_ a file named "Main.pen" with:

```pen
import System'Os

f = \() none {
  f()
}

main = \(os Os'Os) number {
  f()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Run hello world

_Given_ a file named "Main.pen" with:

```pen
import System'Os

main = \(os Os'Os) number {
  Os'FdWrite(os, Os'StdOut(), "Hello, world!\n")

  main(os)
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Create a record

_Given_ a file named "Main.pen" with:

```pen
import System'Os

type foo {
  x number,
}

f = \() none {
  _ = foo{x: 42}

  f()
}

main = \(os Os'Os) number {
  f()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Deconstruct a record

_Given_ a file named "Main.pen" with:

```pen
import System'Os

type foo {
  x number,
}

f = \() none {
  _ = foo{x: 42}.x

  f()
}

main = \(os Os'Os) number {
  f()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Put a string into a value of any type

_Given_ a file named "Main.pen" with:

```pen
import System'Os

f = \(x any) any {
  x
}

g = \() none {
  f("")

  g()
}

main = \(os Os'Os) number {
  g()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Shadow a variable in a block

_Given_ a file named "Main.pen" with:

```pen
import System'Os

type foo {
  x number,
}

f = \() none {
  x = foo{x: 42}
  x = x.x

  f()
}

main = \(os Os'Os) number {
  f()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Define a function in a let expression with a free variable

_Given_ a file named "Main.pen" with:

```pen
import System'Os

type foo {
  x number,
}

f = \() none {
  x = foo{x: 42}
  _ = \() number { x.x }

  f()
}

main = \(os Os'Os) number {
  f()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Convert a number to a string

_Given_ a file named "Main.pen" with:

```pen
import Core'Number
import System'Os

f = \() none {
  Number'String(42)

  f()
}

main = \(os Os'Os) number {
  f()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.

## Join strings

_Given_ a file named "Main.pen" with:

```pen
import Core'String
import System'Os

f = \() none {
  String'Join([string; "hello", "world"])

  f()
}

main = \(os Os'Os) number {
  f()

  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `check_memory_leak_in_loop.sh ./app`.
