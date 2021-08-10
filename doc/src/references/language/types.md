# Types

## Number

```pen
number
```

### Literals

```pen
3.14
-42
```

## Boolean

```pen
boolean
```

### Literals

```pen
false
true
```

## None

```pen
none
```

### Literals

```pen
none
```

## String

```pen
string
```

### Literals

```pen
"foo"
```

## Functions

```pen
\(number, number) number
```

## Lists

```pen
[a]
```

### Literals

```pen
[number; 1, 2, 3]
[myType; x, ...xs]
```

## Records

```pen
type person {
  name string
  age number
}
```

### Literals

- Fields are private outside modules where they are defined.
- Append a suffix of a field name to an expression of a record type to access its value.

```pen
person{name: "foo", age: 42}
person{...john, name: "bar"}
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
