# String

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Use a string literal

_Given_ a file named "Foo.pen" with:

```pen
f = \() string {
  "foo"
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Use equality operators

_Given_ a file named "Foo.pen" with:

```pen
f = \() boolean {
  "" == ""
}

g = \() boolean {
  "" != ""
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
