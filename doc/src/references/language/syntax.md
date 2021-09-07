# Syntax

The language lets you compose programs building up its small constructs. This page describes the syntactical components of the language. See also [Types](types.md) about ones for specific data types.

## Module

Modules are sets of types and functions. One file composes a module. Modules can import functions and types from other modules. See also [Modules](modules.md) to know how modules interact with each other.

A module consists of [statements](#statements).

## Statements

### Import statement

It imports types and functions from another module in the current or another package.

See [Modules](modules.md) for more details.

```pen
import Foo'Bar
```

### Foreign import statement

It imports a function in a foreign language.

You can specify calling convention of foreign languages in a format of string literals after `import foreign` keywords optionally. Currently, only the C calling convention is supported as `"c"`.

See [FFI](/guides/ffi.md) for more details.

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

It defines a new function.

First, followed by a function name (`foo`), it declares its argument names and types (`x number` and `y number`) and its result type (`number`). Then, function bodies describe what the functions do. The function bodies are [blocks](#block).

```pen
foo = \(x number, y number) number {
  x + y
}
```

### Foreign function definition

It defines a new foreign function.

See [FFI](/guides/ffi.md) for more details.

```pen
export foreign foo = \(x number, y number) number {
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

If you want to save results of intermediate expressions and use them somewhere else, you can define variables putting their names and `=` operators in front of the expressions.

```
{
  x = 42

  ...
}
```

## Expressions

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

The operators can be used for any types except functions and types that include them.

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

A not operator flips a boolean value.

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

`?` suffix operator immediately exits the current function with an operand if it is of [the `error` type][error-type].

An operand must be a union type including [the `error` type][error-type].

```
x?
```

[error-type]: built-ins.md#error

### Conditionals

#### If expression

It evaluates one of blocks depending on a value of an expression of a boolean type.

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

It evaluates one of blocks depending on the type of a given expression. The expression (`foo()`) needs to be bound to a variable (`x`) and, in each block, the variable is treated as its specified type.

```pen
if x = foo(); number {
  ...
} else if string | none {
  ...
} else {
  ...
}
```

#### If-list expression

It deconstructs a list and evaluates one of blocks depending on if the list is empty or not.

- If a given list has 1 or more element, it evaluates the first block with a function which returns its first element (`x`) and rest of elements as a list (`xs`).
- If it has no element, it evaluates the second block.

```pen
if [x, ...xs] = ... {
  ...
} else {
  ...
}
```
