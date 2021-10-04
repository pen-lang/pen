# Testing

This page describes how to write and run unit tests for programs written in the language.

The language comes with its built-in test framework. Testing codes consists of the following steps:

1. Add the `Test` package in package configuration.
1. Write tests as functions in test modules.
1. Run the tests with a `pen test` command.

## Writing tests

Tests are expressed as functions in test modules. Test modules are all the modules that have the `.test.pen` file extension. The functions need to be public to be recognized as test functions.

For example, to test a `Foo` function in a `Foo.pen` module, write a `Foo.test.pen` test module with the following contents.

```pen
import Test'Assert
import 'Foo

CheckFoo = \() none | error {
  Assert'True(Foo(42, "foo"))
}
```

## Running tests

To run those tests, you can run a `pen test` command in your terminal. Then, you should see test results of each functions in each modules.
