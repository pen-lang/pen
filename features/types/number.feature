Feature: Number
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Use a number literal
    Given a file named "Foo.pen" with:
      """pen
      f = \() number {
        42
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use arithmetic operators
    Given a file named "Foo.pen" with:
      """pen
      f = \() number {
        1 + 2 - 3 * 4 / 5
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use equality operators
    Given a file named "Foo.pen" with:
      """pen
      f = \() boolean {
        0 == 0
      }

      g = \() boolean {
        0 != 0
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use order operators
    Given a file named "Foo.pen" with:
      """pen
      f = \() boolean {
        0 < 0
      }

      g = \() boolean {
        0 <= 0
      }

      h = \() boolean {
        0 > 0
      }

      i = \() boolean {
        0 >= 0
      }
      """
    When I run `pen build`
    Then the exit status should be 0
