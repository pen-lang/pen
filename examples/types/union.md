# Union

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Use a union type

_Given_ a file named "Foo.pen" with:

```pen
f = \() number | none {
  42
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Downcast a union type

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number | none) number {
  if x = x; number {
    x
  } else if none {
    0
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Downcast a union type with an else block

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number | none) number {
  if x = x; none {
    0
  } else {
    x
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Downcast a union type to another union type

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number | boolean | none) number | none {
  if x = x; number | none {
    x
  } else {
    none
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
