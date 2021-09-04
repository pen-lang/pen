# Types

## Number

It is a 64-bit floating point number of [IEEE 754](https://en.wikipedia.org/wiki/IEEE_754).

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

List literals contain elements or other lists prefixed by `...` which are joined into the lists. Element types need to be specified explicitly before semicolons like `[number; ... ]`.

Expressions within list literals are evaluated lazily; they are evaluated only if their values are needed.

```pen
[number; 1, 2, 3]
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

Record values are constructed by their literals containing their field names and values.

```pen
person{name: "foo", age: 42}
```

You can also create new records from existing ones spreading fields of the old records into the literals.

```pen
person{...john, name: "bar"}
```

You can append field names prefixed by `.` to expressions of record types to access their values.

```pen
john.name
```

## Singleton records

```pen
type foo {}
```

### Literals

- Their values can be referenced as variables of their names.

```pen
foo
```

## Unions

- Unions combine different types into a type.

```pen
foo | bar
```

## Any

- `any` is something called a "top" type.
- Any types can be casted to `any` type.

```pen
any
```
