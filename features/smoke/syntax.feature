Feature: Syntax
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Override a built-in function
    Given a file named "Foo.pen" with:
      """pen
      go = \() none {
        none
      }

      foo = \() none {
        go()
      }
      """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Override a built-in type with a type definition
    Given a file named "Foo.pen" with:
      """pen
      type none {
        x number
      }

      foo = \() none {
        none{x: 42}
      }
      """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Override a built-in type with a type alias
    Given a file named "Foo.pen" with:
      """pen
      type none = number

      foo = \() none {
        0
      }
      """
    When I successfully run `pen build`
    Then the exit status should be 0
