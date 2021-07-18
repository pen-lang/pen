---
title: Syntax
---

# Syntax

## Module

A module consists of [statements](#statements).

## Statements

### Import statement

See [Modules](../modules).

### Foreign import statement

```pen
import foreign "c" foo \(number, number) number
```

### Record type definition

See [Records](../types#records).

### Type alias

It gives another name to a given type.

```pen
type foo = number | none
```

### Function definition

- Followed by a function name, it declares its argument names and types in order and its result type.
- Bodies of functions are [blocks](#block).

```pen
foo = \(x number, y number) number {
  x + y
}
```

## Block

- It consists of 0 or more vaiable bindings and an expression.

```pen
{
  z = x + y
  ...
  foo(ctx, z)
  ...

  x + y + z
}
```

## Expressions

### Operators

#### Arithmetic

```pen
1 + 1
1 - 1
1 * 1
1 / 1
```

#### Comparison

```pen
1 == 1
1 != 1
1 < 1
1 <= 1
1 > 1
1 >= 1
```

##### Generic equality

`==` and `!=` operators can be used for any types except functions and types that include them.

```pen
"foo" == "bar"
foo{x: 0} == foo{x: 1}
42 != none
```

#### Boolean

```pen
!true
true & false
true | false
```

#### Error handling

- The `?` suffix operator immediately exits the current function with an operand's value if it is of [the `error` type][error-type].
- An operand must be a union type including [the `error` type][error-type].

```
x?
```

[error-type]: ../built-ins#error

### Function call

```pen
f(x, y)
```

### Conditionals

#### `if` expression

```pen
if x {
  ...
} else {
  ...
}
```

#### `if`-type expression

- It evaluates one of blocks depending on types of a given expressions bound to a given name.
- In each block, the given name is bound to a variable of the specified type.

```pen
if x = ...; number {
  ...
} else if string | none {
  ...
} else {
  ...
}
```

#### `if`-list expression

- It evaluates one of blocks depending on values of given lists.
- If a given list has 1 or more element, it evaluates the first block with its first element and rest of elements as a list.
- If it has no element, it evaluates the second block.

```pen
if [x, ...xs] = ... {
  ...
} else {
  ...
}
```
