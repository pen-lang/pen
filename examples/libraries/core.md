# Core

## Background

_Given_ a file named "pen.json" with:

```json
{
  "dependencies": {
    "Core": "file://pen-root/lib/core"
  }
}
```

## Convert a number to a string

_Given_ a file named "Foo.pen" with:

```pen
import Core'Number

f = \() string {
  Number'String(42)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Join strings

_Given_ a file named "Foo.pen" with:

```pen
import Core'String

f = \() string {
  String'Join([string; "hello", "world"])
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Slice a string

_Given_ a file named "Foo.pen" with:

```pen
import Core'String

f = \() string {
  String'Slice("foo", 1, 2)
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
