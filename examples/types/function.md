# Function

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Define a function

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number) number {
  x
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Call a function with no argument

_Given_ a file named "Foo.pen" with:

```pen
f = \() number {
  f()
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Call a function with an argument

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number) number {
  f(x)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Call a function with two arguments

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number, y number) number {
  f(x, y)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Define a closure

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number) \(number) number {
  \(y number) number {
    x + y
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
