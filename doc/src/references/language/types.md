# Types

This page describes different data types in Pen.

## Number

It represents a real number. It is implemented as a 64-bit floating point number of [IEEE 754](https://en.wikipedia.org/wiki/Double-precision_floating-point_format).

```pen
number
```

### Literals

```pen
3.14
-42
```

## Boolean

It is a boolean value of `true` or `false`.

```pen
boolean
```

### Literals

```pen
true
false
```

## None

It represents a missing value. It has only a single value of `none`.

```pen
none
```

### Literals

```pen
none
```

## String

It is a byte array.

```pen
string
```

### Literals

String literals are sequences of bytes. They are often used to represent texts encoded in [UTF-8](https://en.wikipedia.org/wiki/UTF-8).

```pen
"foo"
```

#### Escape sequences

String literals can contain the following escape sequences.

| Escape sequence | Name            |
| --------------- | --------------- |
| `\n`            | New line        |
| `\r`            | Carriage return |
| `\t`            | Tab             |
| `\"`            | Double quote    |
| `\\`            | Backslash       |
| `\x9f`          | Byte            |

## Functions

A function represents reusable computation with arguments and a result.

Functions represent not only pure computation but may also execute side effects, such as I/O.

```pen
\(number, number) number
```

### Literals

```pen
\(x number, y number) number {
  x + y
}
```

## Lists

It is a list of values. Its type contains an element type between `[` and `]`.

```pen
[number]
```

### Literals

A list literal contains its element type and elements as expressions.

```pen
[number]
[number 1]
[number 1, 2, 3]
```

You can create new lists from existing ones by spreading elements of the old ones prefixed by `...` into the new ones.

```pen
[number x, ...xs]
```

Note that **expressions within list literals are evaluated lazily**; they are evaluated only if their values are required.

## Maps

It is a map from keys to values. Its type contains key and value types between `{` and `}` separated by `:`.

```pen
{string: number}
```

### Literals

A map literal contains its key and value types and key-value pairs as expressions.

```pen
{string: number}
{string: number "foo": 1}
{string: number "foo": 1, "bar": 2}
```

You can create new maps from existing ones by spreading entries of the old ones prefixed by `...` into the new ones.

```pen
{string: number ...map, "foo": 1}
```

You can also delete a key from a map omitting its value.

```pen
{string: number ...map, "foo"}
```

## Records

It combines multiple types into a single type. Each field of a record type is composed of its name and type.

Fields are not accessible outside modules where they are defined by default.

```pen
type person {
  name string
  age number
}
```

To expose fields as well as the type itself to other modules, you need to capitalize their names.

```pen
type Person {
  Name string
  Age number
}
```

### Literals

Record values are constructed using record literals containing their field names and values separated by commas.

```pen
person{name: "foo", age: 42}
```

You can also create new records from existing ones spreading fields of the old ones into the new ones.

```pen
person{...one, name: "bar"}
```

You can access field values by appending their names with `.` prefixes to expressions of record types.

```pen
john.name
```

## Unions

It is a union of multiple types.

For example, the type below represents values that can be either `number` or `none`.

```pen
number | none
```

## Any

Literally, it's an _any_ type. Any values can be converted to the type.

```pen
any
```

## Error

It is an error. See also [Error handling](syntax.md#error-handling).

You can create error values by calling [the `error` built-in function](built-ins.md#error).

```pen
error
```
