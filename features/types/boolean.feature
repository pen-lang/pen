Feature: Boolean
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Use boolean literals
    Given a file named "Foo.pen" with:
      """pen
      f = \() boolean {
        true
      }

      g = \() boolean {
        false
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use an and operation
    Given a file named "Foo.pen" with:
      """pen
      f = \() boolean {
        true & false
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use an or operation
    Given a file named "Foo.pen" with:
      """pen
      f = \() boolean {
        true | false
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use a not operation
    Given a file named "Foo.pen" with:
      """pen
      f = \() boolean {
        !true
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use an if expression
    Given a file named "Foo.pen" with:
      """pen
      f = \() number {
        if true {
          1
        } else {
          0
        }
      }
      """
    When I run `pen build`
    Then the exit status should be 0
