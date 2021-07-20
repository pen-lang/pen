# None

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Use a none literal

_Given_ a file named "Foo.pen" with:

```pen
f = \() none {
  none
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
