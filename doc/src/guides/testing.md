# Testing

This page describes how to write and run unit tests for programs written in the language.

Testing codes consists of the following steps:

1. Add the `Test` package in package configuration.
1. Write tests as _test_ functions in _test_ modules.
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

The `Test` package also includes some utilities which helps you to write tests. See [its reference](/references/standard-packages/test.html) for more information.

## Writing tests

You can write tests as _test_ functions in _test_ modules. All modules with `.test.pen` file extensions are test modules. And, all public functions in test modules are test functions. The test functions need to have a type of `\() none | error` and should return `error` values when they fail. A `pen test` command described later runs the test functions as tests.

For example, to test a `Foo` function in a `Foo.pen` module, write a `Foo.test.pen` test module with the following contents.

```pen
import Test'Assert
import 'Foo

CheckFoo = \() none | error {
  Assert'True(Foo(42, "foo"))
}
```

## Running tests

To run tests, you can run a `pen test` command in your package's directory. Then, you should see test results of each functions in each modules. The `pen test` command exits with non-zero status codes if some tests fail.
