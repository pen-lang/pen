# Polymorphism

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Use an equal operator

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  42 == none
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use a not-equal operator

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  42 != none
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Compare unions

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number | none, y number | none) boolean {
  x == y
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Compare a union and none

_Given_ a file named "Foo.pen" with:

```pen
f = \(x number | none) boolean {
  x == none
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
