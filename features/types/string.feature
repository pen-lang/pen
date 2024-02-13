Feature: String
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Use a string literal
    Given a file named "Foo.pen" with:
      """pen
      f = \() string {
        "foo"
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use equality operators
    Given a file named "Foo.pen" with:
      """pen
      f = \() boolean {
        "" == ""
      }

      g = \() boolean {
        "" != ""
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Concatenate strings
    Given a file named "Foo.pen" with:
      """pen
      f = \() string {
        "foo" + "bar"
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Concatenate 3 strings
    Given a file named "Foo.pen" with:
      """pen
      f = \() string {
        "foo" + "bar" + "baz"
      }
      """
    When I run `pen build`
    Then the exit status should be 0
