Feature: Packages
  Background:
    Given a file named "foo/pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """
    And a file named "foo/Foo.pen" with:
      """pen
      type Foo = number

      Foo = \() number {
        42
      }
      """
    And a file named "foo/Foo/Foo.pen" with:
      """pen
      Foo = \() number {
        42
      }
      """
    And a directory named "bar"
    And I cd to "bar"
    And a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {
          "Foo": "../foo"
        }
      }
      """

  Scenario: Import a function from a module
    Given a file named "Bar.pen" with:
      """pen
      import Foo'Foo

      Bar = \() number {
        Foo'Foo()
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Import a type from a module
    Given a file named "Bar.pen" with:
      """pen
      import Foo'Foo

      type Bar = Foo'Foo
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Import a function from a nested module
    Given a file named "Bar.pen" with:
      """pen
      import Foo'Foo'Foo

      Bar = \() number {
        Foo'Foo()
      }
      """
    When I run `pen build`
    Then the exit status should be 0
