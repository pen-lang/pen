Feature: Modules
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Import a duplicate name
    Given a file named "foo.pen" with:
      """pen
      Foo = \(x number) number { x }
      """
    And a file named "bar.pen" with:
      """pen
      import 'foo { Foo }

      Foo = \() none { Foo() }
      """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Compare a type imported indirectly
    Given a file named "foo.pen" with:
      """pen
      type Foo {}
      """
    And a file named "bar.pen" with:
      """pen
      import 'foo { Foo }

      type Bar {
        xs [Foo]
      }
      """
    And a file named "baz.pen" with:
      """pen
      import 'bar { Bar }

      type Baz = Bar
      """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Import an undefined name
    Given a file named "foo.pen" with:
      """pen
      """
    And a file named "bar.pen" with:
      """pen
      import 'foo { tomato }
      """
    When I run `pen build`
    Then the exit status should not be 0
