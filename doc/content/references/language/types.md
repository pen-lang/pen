---
title: Types
---

# Types

## Number

```
number
```

### Literals

```
3.14
-42
```

## Boolean

```
boolean
```

### Literals

```
false
true
```

## None

```
none
```

### Literals

```
none
```

## String

```
string
```

### Literals

```
"foo"
```

## Functions

```
\(number, number) number
```

## Lists

```
[a]
```

### Literals

```
[number; 1, 2, 3]
[myType; x, ...xs]
```

## Records

```
type person {
  name string,
  age number,
}
```

### Literals

- Fields are private outside modules where they are defined.
- Append a suffix of a field name to an expression of a record type to access its value.

```
person{name: "foo", age: 42}
person{...john, name: "bar"}
john.name
```

## Singletons

```
type foo {}
```

### Literals

- Singleton values can be referenced by their type names.

```
foo
```

## Unions

- Unions combine different types into a type.

```
foo | bar
```

## Any

- `any` is something called a "top" type.
- Any types can be casted to `any` type.

```
any
```
