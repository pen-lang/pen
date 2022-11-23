# Syntax

This page describes syntax of Pen. You can compose programs building up those language constructs. See also [Types](types.md) about syntax for specific data types.

## Module

Modules are sets of type and function definitions. Syntactically, a module consists of [statements](#statements). See [Modules](modules.md) about how modules themselves interact with each other.

## Statements

Statements are constructs that declare functions and types in modules.

### Import statement

It imports types and functions from another module from the same or another package.

See [Modules](modules.md) for more details.

```pen
import Foo'Bar
```

### Foreign import statement

It imports a function from a foreign language.

See [Foreign Function Interface (FFI)](../../advanced-features/ffi.md) for more details.

```pen
import foreign "c" foo \(number, number) number
```

### Record type definition

It defines a record type.

See [Records](types.md#records) for more details.

```pen
type foo {
  bar number
  baz string
}
```

### Type alias

It gives another name to a type.

```pen
type foo = number | none
```

### Function definition

It defines a function with a given name. The right-hand side of `=` signs must be [function expressions](#function).

```pen
foo = \(x number, y number) number {
  x + y
}
```

### Foreign function definition

It defines a function exported to foreign languages.

See [Foreign Function Interface (FFI)](../../advanced-features/ffi.md) for more details.

```pen
foreign "c" foo = \(x number, y number) number {
  x + y
}
```

## Block

A block consists of 1 or more expressions wrapped in `{` and `}`. Values of the last expressions are treated as resulting values of the blocks.

```pen
{
  foo(ctx, z)

  x + y + z
}
```

If you want to keep values of intermediate expressions for later use, you can define variables putting their names and `=` operators in front of the expressions.

```pen
{
  x = 42

  ...
}
```

## Expressions

Expressions represent some computation. Expressions can be nested; expressions often contain other expressions inside.

### Function call

It calls a function to evaluate it with given arguments returning its result value.

```pen
f(x, y)
```

### Operators

#### Arithmetic

Arithmetic operators add, subtract, multiply, or divide a number with another.

```pen
1 + 1
1 - 1
1 * 1
1 / 1
```

#### Comparison

##### Equality

Equal (`==`) and not-equal (`!=`) operators compare two values and return a boolean value indicating if they are equal or not.

```pen
1 == 1
1 != 1
```

The operators can compare any types except functions and types containing them.

```pen
"foo" == "bar"
foo{x: 0} == foo{x: 1}
42 != none
```

##### Ordering

Order operators compare two numbers and return a boolean value indicating if the condition is correct or not.

```pen
1 < 1
1 <= 1
1 > 1
1 >= 1
```

#### Boolean

A _not_ operator flips a boolean value.

```pen
!true
```

An _and_ operator returns `true` if both operands are `true`, or `false` otherwise.

```pen
true & false
```

An _or_ operator returns `true` if either operand is `true`, or `false` otherwise.

```pen
true | false
```

#### Error handling

`?` suffix operators immediately exit the current functions with operands if they are of [the `error` type][error-type]. Both the operands and result values of functions where the operators are used must be a union type containing [the `error` type][error-type].

```pen
x?
```

[error-type]: built-ins.md#error

### Function

It creates a function.

First, functions declare their argument names and types (`x number` and `y number`) and their result types (`number`). After that, function bodies of [blocks](#block) describe how the functions compute result values.

```pen
\(x number, y number) number {
  x + y
}
```

### Conditionals

#### If expression

It evaluates one of [blocks](#block) depending on a condition of an expression of a boolean type.

- It evaluates the first block if a given boolean value is `true`.
- Otherwise, it evaluates the second block.

```pen
if x {
  ...
} else {
  ...
}
```

#### If-type expression

It evaluates one of [blocks](#block) depending on the type of a given expression. The expression (`foo()`) needs to be bound to a variable (`x`) and, in each block, the variable is treated as its specified type.

```pen
if x = foo() as number {
  ...
} else if string | none {
  ...
} else {
  ...
}
```

#### If-list expression

It deconstructs a list and evaluates one of two [blocks](#block) depending on if the list is empty or not.

- If a given list has 1 or more element, it evaluates the first block with **a function that returns its first element** (`x`) and rest of elements as a list (`xs`).
- If the list has no element, it evaluates the second block.

```pen
if [x, ...xs] = ... {
  ...
} else {
  ...
}
```

#### If-map expression

It gets a value for a key in a map and evaluates one of two [blocks](#block) depending on if the map has the key or not.

- If a value for a key (`key`) is found, it evaluates the first block with the value (`value`).
- If the map has no such key, it evaluates the second block.

```pen
if value = xs[key] {
  ...
} else {
  ...
}
```

### Loop

#### List comprehension

It iterates over elements in a given list and creates a new list with elements of a given expression.

```pen
[number f(x()) for x in xs]
```

You can iterate key-value pairs in a map.

```pen
[number f(key, value) for key, value in map]
```

You can use multiple `for` clauses to iterate multiple lists and maps.

```pen
[number f(y())
  for y in x()
  for x in xs
]
```

You can use `if` clauses to filter elements.

```pen
[number f(x()) for x in xs if g(x())]
```

## Comment

Comments start with `#` and end with new-line characters.

```pen
# This is a comment.
```
