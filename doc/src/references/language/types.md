# Types

This page describes different data types in the language.

## Number

It represents a real number. It is implemented as a 64-bit floating point number of [IEEE 754](https://en.wikipedia.org/wiki/Double-precision_floating-point_format) under the hood.

```pen
number
```

### Literals

```pen
3.14
-42
```

## Boolean

It is a boolean value of `true` or `false` denoting if a statement is correct or not.

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

They represent texts encoded in [UTF-8](https://en.wikipedia.org/wiki/UTF-8).

```pen
"foo"
```

## Functions

A function is a set of operations with a list of arguments and a result.

Functions represent not only pure computation but also "routines" which might execute side effects, such as I/O.

```pen
\(number, number) number
```

## Lists

It is a list of values of a type. The element type is put between `[` and `]`.

```pen
[number]
```

### Literals

A list literal contains its element type and elements as expressions.

Expressions within list literals are evaluated lazily; they are evaluated only if their values are required but only once.

```pen
[number]
[number 1]
[number 1, 2, 3]
```

You can create new lists from existing ones by spreading elements of the old ones prefixed by `...` into the new ones.

```pen
[person x, ...xs]
```

## Records

It is a combination of types banded into a single type. Each field of a record type is composed of its name and type.

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

Record values are constructed using record literals containing their field names and values.

```pen
person{name: "foo", age: 42}
```

You can also create new records from existing ones spreading fields of the old ones into the new ones.

```pen
person{...john, name: "bar"}
```

You can access field values by appending their names with `.` prefixes to expressions of record types.

```pen
john.name
```

## Singleton records

Singleton records are special record types which have no field and, therefore, only one kind of values.

```pen
type foo {}
```

### Literals

Their values are accessible as variables of their names.

```pen
foo
```

## Unions

It is a segregated union of types.

For example, the type below represents values which can be either `number` or `none`. But values of the type cannot be `number` and `none` types at the same time.

```pen
number | none
```

## Any

Literally, it's an _any_ type. Any values can be converted to the type.

```pen
any
```
