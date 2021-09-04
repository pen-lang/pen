# Types

## Number

It represents a real number. It is implemented as a 64-bit floating point number of [IEEE 754](https://en.wikipedia.org/wiki/IEEE_754) under the hood.

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

It represents a missing value. It has only a single literal of `none`.

```pen
none
```

### Literals

```pen
none
```

## String

It represents texts encoded in [UTF-8](https://en.wikipedia.org/wiki/UTF-8) or byte arrays.

```pen
string
```

### Literals

```pen
"foo"
```

## Functions

It is a function with a list of arguments and a result.

In the language, functions represent not only pure mathematical ones but also "routines" which execute side effects, such as I/O.

```pen
\(number, number) number
```

## Lists

It is a list of values of some type. The element type is put between `[` and `]`.

```pen
[number]
```

### Literals

List literals contain their elements. Element types need to be specified explicitly before semicolons like `[number; ... ]`.

Expressions within list literals are evaluated lazily; they are evaluated only if their values are required.

```pen
[number; 1, 2, 3]
```

You can create new lists from existing ones by spreading elements of the old ones prefixed by `...` into the new ones.

```pen
[person; x, ...xs]
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

### Literals

Record values are constructed using record literals containing their field names and values.

```pen
person{name: "foo", age: 42}
```

You can also create new records from existing ones spreading fields of the old ones into the literals.

```pen
person{...john, name: "bar"}
```

You can access field values by appending their names with `.` prefixes to expressions of the record types.

```pen
john.name
```

## Singleton records

Singleton records are special record types which have only one kind of values.

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

For example, the type below represents one of values which can be either `number` or `none`. But values of the type cannot be `number` and `none` types at the same time.

```pen
number | none
```

## Any

Literally, it's an _any_ type. Any values can be converted to the type.

```pen
any
```
