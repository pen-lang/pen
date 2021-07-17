# Syntax

## Statements

Variables and functions can be defined using the `:` sign to specify their types and the `=` sign to specify their values.

### Variable definition

```
x : Number
x = ...
```

### Function definition

```
f : Number -> Number -> Number
f x y = ...
```

### Record type definition

See [Records](types.md#records).

### Type alias

```
type Foo = ...
```

### Module import and export

See [Modules](modules.md).

## Expressions

### Operators

#### Arithmetic

```
1 + 1
1 - 1
1 * 1
1 / 1
```

#### Comparison

```
1 == 1
1 /= 1
1 < 1
1 <= 1
1 > 1
1 >= 1
```

##### Generic equality

`==` and `/=` operators can be used for any types except functions and types which might include them.

```
"foo" == "bar"
Foo{ foo : 0 } == Foo{ foo : 1 }
42 /= None
```

#### Boolean

```
True && True
True || True
```

### Function application

```
f x
```

### Conditionals

#### `if` expression

```
if x then
  ...
else
  ...
```

#### `case` expression

##### Lists

```
case xs
  [] => ...
  [ y, ...ys ] => ...
```

##### Unions and `Any`

- Values of union and `Any` types can be downcasted using the `case` expression.
- The variable (`x` in the code below) is bound as a different type in each branch.

```
case x = ...
  Foo => ...
  Bar | Baz => ...
```

### Bindings

#### `let` expression

- Both variable and function definitions can be used in the `let` expressions.

```
let
  x = 1
  f x = x + 1
in
  x + f y
```

#### `let`-error expression

- Using `case` expressions for error handling is hard because the expressions often get nested deeply.
- `let`-error expression flattens those error handlings by propagating errors in variable definitions to a value of the whole expression.

Given `x : Number | Error`,

```
let
  y ?= x
in
  y + 1
```

is equivalent to

```
case y = x
  Error => y
  Number => y + 1
```
