# Syntax

This page describes the syntactical components of the language. You can compose programs building up those constructs. See also [Types](types.md) about syntax for specific data types.

## Module

Modules are sets of types and functions. See [Modules](modules.md) for more information.

Syntactically, a module consists of [statements](#statements).

## Statements

Statements are constructs that declare functions and types in modules.

### Import statement

It imports types and functions from another module in the same or another package.

See [Modules](modules.md) for more details.

```pen
import Foo'Bar
```

### Foreign import statement

It imports a function in a foreign language.

See [FFI](/advanced-features/ffi.md) for more details.

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

It defines a function with a given name. The right hand side of `=` signs must be [function expressions](#function).

```pen
foo = \(x number, y number) number {
  x + y
}
```

### Foreign function definition

It defines a function exported for foreign languages.

See [FFI](/advanced-features/ffi.md) for more details.

```pen
foreign foo = \(x number, y number) number {
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

If you want to save results of intermediate expressions for later use, you can define variables putting their names and `=` operators in front of the expressions.

```pen
{
  x = 42

  ...
}
```

## Expressions

Expressions express what programs actually compute. Notably, expressions can be nested; many expressions contain other expressions inside.

### Function call

It calls a function to evaluate it with given arguments returning its result value.

```pen
f(x, y)
```

### Operators

#### Arithmetic

Arithmetic operators add, subtract, multiply or divide a number with another one.

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

The operators can compare any types except functions and types that include them.

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

`?` suffix operator immediately exits the current function with an operand if it is of [the `error` type][error-type]. The operand must be a union type containing [the `error` type][error-type].

```pen
x?
```

[error-type]: built-ins.md#error

### Function

It creates a function.

First, functions declare their argument names and types (`x number` and `y number`) and their result types (`number`). After that, function bodies describe what the functions do as [blocks](#block).

```pen
\(x number, y number) number {
  x + y
}
```

### Conditionals

#### If expression

It evaluates one of [blocks](#block) depending on a value of an expression of a boolean type.

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

It deconstructs a list and evaluates one of [blocks](#block) depending on if the list is empty or not.

- If a given list has 1 or more element, it evaluates the first block with a function which returns its first element (`x`) and rest of elements as a list (`xs`).
- If it has no element, it evaluates the second block.

```pen
if [x, ...xs] = ... {
  ...
} else {
  ...
}
```

## Comment

Comments start with `#` and end with new-line characters.

```pen
# This is an important comment.
```
