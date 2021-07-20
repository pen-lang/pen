# Building packages

## Build an application package

_Given_ a file named "pen.json" with:

```json
{
  "dependencies": {
    "System": "file://pen-root/lib/os"
  }
}
```

_And_ a file named "Main.pen" with:

```pen
import System'Os

main = \(os Os'Os) number {
  0
}
```

_When_ I successfully run `pen build`

_Then_ I successfully run `./app`.

## Build a library package

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

_And_ a file named "Foo.pen" with:

```pen
f = \(x number) number {
  x
}
```

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Cross-build a library package

_Given_ a file named "pen.json" with:

```json
{ "dependencies": {} }
```

_And_ a file named "Foo.pen" with:

```pen
f = \(x number) number {
  x
}
```

_When_ I run `pen build --target wasm32-unknown-unknown`

_Then_ the exit status should be 0.
