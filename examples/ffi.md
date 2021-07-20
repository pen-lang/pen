# FFI

## Background

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

## Import a foreign function of native calling convention

_Given_ a file named "Foo.pen" with:

```pen
import foreign g \(number) number

f = \(x number) number {
  g(x)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Import a foreign function of C calling convention

_Given_ a file named "Foo.pen" with:

```pen
import foreign "c" g \(number) number

f = \(x number) number {
  g(x)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Export a foreign function

_Given_ a file named "Foo.pen" with:

```pen
export foreign f = \(x number) number {
  x
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
