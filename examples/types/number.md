# Number

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Use a number literal

_Given_ a file named "Foo.pen" with:

```pen
f = \() number {
  42
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use arithmetic operators

_Given_ a file named "Foo.pen" with:

```pen
f = \() number {
  1 + 2 - 3 * 4 / 5
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use equality operators

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  0 == 0
}

g = \() boolean {
  0 != 0
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use order operators

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  0 < 0
}

g = \() boolean {
  0 <= 0
}

h = \() boolean {
  0 > 0
}

i = \() boolean {
  0 >= 0
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
