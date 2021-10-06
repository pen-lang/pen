# Testing

This page describes how to write and run unit tests for programs written in the language.

Testing codes consists of the following steps:

1. Add the `Test` package in package configuration.
1. Write tests as functions in _test_ modules.
1. Run the tests with a `pen test` command.

## The `Test` package

Before writing any tests, you need to add the `Test` standard package in your package's [configuration file](/references/language/packages.html#package-configuration) as follows.

```json
{
  "dependencies": {
    ...
    "Test": "pen:///lib/test",
    ...
  }
}
```

The `Test` package also includes some utilities which helps you to write tests. See [`Test` package](/references/standard-packages/test.html) for more information.

## Writing tests

You can write tests as functions in _test_ modules. Test modules are all the modules that have the `.test.pen` file extension. The functions need to be public to be recognized as test functions.

For example, to test a `Foo` function in a `Foo.pen` module, write a `Foo.test.pen` test module with the following contents.

```pen
import Test'Assert
import 'Foo

CheckFoo = \() none | error {
  Assert'True(Foo(42, "foo"))
}
```

## Running tests

To run tests, you can run a `pen test` command in your terminal. Then, you should see test results of each functions in each modules.
