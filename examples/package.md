# Packages

## Background

_Given_ a file named "foo/pen.json" with:

```json
{ "dependencies": {} }
```

_And_ a file named "foo/Foo.pen" with:

```pen
Foo = \() number {
  42
}
```

_And_ a file named "foo/Foo/Foo.pen" with:

```pen
Foo = \() number {
  42
}
```

_And_ a directory named "bar"

_And_ I cd to "bar"

_And_ a file named "pen.json" with:

```json
{
  "dependencies": {
    "Foo": "file+relative:../foo"
  }
}
```

## Import a function from a module

_Given_ a file named "Bar.pen" with:

```pen
import Foo'Foo

Bar = \() number {
  Foo'Foo()
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Import a type alias from a module

_Given_ a file named "Bar.pen" with:

```pen
import Foo'Foo

type Bar = Foo'Foo
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Import a function from a nested module

_Given_ a file named "Bar.pen" with:

```pen
import Foo'Foo'Foo

Bar = \() number {
  Foo'Foo()
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.
