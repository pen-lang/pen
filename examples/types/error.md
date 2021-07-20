# Error

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Call a error function

_Given_ a file named "Foo.pen" with:

```pen
f = \() error {
  error(none)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Call a source function

_Given_ a file named "Foo.pen" with:

```pen
f = \(e error) any {
  source(e)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use a try operator

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number | error) number | error {
  x? + 1
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use a try operator with a union type

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number | none | error) number | error {
  if x = x?; number {
    x + 1
  } else if none {
    0
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
