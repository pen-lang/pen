# Block

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Define a variable

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number) number {
  y = x

  y
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Call a function

_Given_ a file named "Foo.pen" with:

```pen
f = \() none {
  none
}

g = \() none {
  f()

  none
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use if expression

_Given_ a file named "Foo.pen" with:

```pen
f = \() none {
  none
}

g = \() none {
  none
}

h = \(x boolean) none {
  if x {
    f()
  } else {
    g()
  }

  none
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
