# Types

## Number

```
Number
```

### Literals

```
3.14
-42
```

## Boolean

```
Boolean
```

### Literals

```
False
True
```

## None

```
None
```

### Literals

```
None
```

## String

```
String
```

### Literals

```
"foo"
```

## Functions

```
a -> b
```

## Lists

```
[a]
```

### Literals

```
[ 1, 2, 3 ]
[ x, ...xs ]
```

## Records

```
type Person {
  name : String,
  age : Number,
}
```

### Literals

- Fields are private outside modules where they are defined.

```
Person.name person
Person{ name = "foo", age = 42 }
Person{ ...person, name = "bar" }
```

## Singletons

```
type Foo
```

### Literals

- Singleton values can be referenced by their type names.

```
Foo
```

## Unions

- Unions combine different types into a type.

```
Foo | Bar
```

## Any

- `Any` is something called a "top" type.
- Any types can be casted to `Any` type.

```
Any
```
