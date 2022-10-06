# Testing

This page describes how to write and run unit tests for programs written in Pen.

Testing codes consists of the following steps:

1. Write tests as _test_ functions in _test_ modules.
1. Run the tests with a `pen test` command.

## Writing tests

You can write tests as _test_ functions in _test_ modules. All modules with the `.test.pen` file extension are test modules. And, all public functions in test modules are test functions. The test functions need to have a type of `\() none | error` and should return `error` values when they fail.

For example, to test a `Foo` function in a `Foo.pen` module, write a `Foo.test.pen` test module with the following contents.

```pen
import Test'Assert
import 'Foo

CheckFoo = \() none | error {
  Assert'Equal(Foo'Foo(), 42)
}
```

### The `Test` package

[The `Test` standard package](../references/standard-packages/test.md) includes some utilities which helps you to write tests.

## Running tests

To run tests, you can run a `pen test` command in your package's directory. Then, you should see test results of test functions in test modules. The `pen test` command exits with a non-zero status code if some tests fail.
