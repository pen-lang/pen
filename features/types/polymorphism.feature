Feature: Polymorphism
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Use an equal operator
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number | none) boolean {
        x == none
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use a not-equal operator
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number | none) boolean {
        x != none
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Compare unions
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number | none, y number | none) boolean {
        x == y
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Compare a union and none
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number | none) boolean {
        x == none
      }
      """
    When I run `pen build`
    Then the exit status should be 0
