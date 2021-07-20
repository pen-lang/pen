# Any

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Use an any type

_Given_ a file named "Foo.pen" with:

```pen
f = \() any {
  42
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Downcast an any type

_Given_ a file named "Foo.pen" with:

```pen
f = \(x any) number {
  if x = x; number {
    x
  } else {
    0
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
