# Record

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Create a record with an element

_Given_ a file named "Foo.pen" with:

```pen
type r {
  x number,
}

f = \() r {
  r{x: 42}
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Create a record with two elements

_Given_ a file named "Foo.pen" with:

```pen
type r {
  x number,
  y none,
}

f = \() r {
  r{x: 42, y: none}
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Create a record with no element

_Given_ a file named "Foo.pen" with:

```pen
type r {}

f = \() r {
  r
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Update a record

_Given_ a file named "Foo.pen" with:

```pen
type r {
  x number,
  y none,
}

f = \(x r) r {
  r{...x, y: none}
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Get an elemnt in a record

_Given_ a file named "Foo.pen" with:

```pen
type r {
  x number,
}

f = \(x r) number {
  x.x
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use an equal operator

_Given_ a file named "Foo.pen" with:

```pen
type r {
  x number,
}

f = \(x r, y r) boolean {
  x == y
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use a not-equal operator

_Given_ a file named "Foo.pen" with:

```pen
type r {
  x number,
}

f = \(x r, y r) boolean {
  x == y
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
