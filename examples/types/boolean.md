# Boolean

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Use boolean literals

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  true
}

g = \() boolean {
  false
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use an and operation

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  true & false
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use an or operation

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  true | false
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use a not operation

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  !true
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use an if expression

_Given_ a file named "Foo.pen" with:

```pen
f = \() number {
  if true {
    1
  } else {
    0
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
