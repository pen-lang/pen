# List

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Create an empty list

_Given_ a file named "Foo.pen" with:

```pen
f = \() [number] {
  [number;]
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Create a list with an element

_Given_ a file named "Foo.pen" with:

```pen
f = \() [number] {
  [number; 1]
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Create a list with two elements

_Given_ a file named "Foo.pen" with:

```pen
f = \() [number] {
  [number; 1, 2]
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Join lists

_Given_ a file named "Foo.pen" with:

```pen
f = \(xs [number]) [number] {
  [number; ...xs, ...xs]
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Create a list of a union type

_Given_ a file named "Foo.pen" with:

```pen
f = \() [number | none] {
  [number | none; 1, none]
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Coerce elements of a spraed list

_Given_ a file named "Foo.pen" with:

```pen
f = \(xs [number]) [number | none] {
  [number | none; ...xs]
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use if-list expression

_Given_ a file named "Foo.pen" with:

```pen
f = \(xs [number]) [number] {
  if [y, ...ys] = xs {
    [number; y, ...ys]
  } else {
    [number;]
  }
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
