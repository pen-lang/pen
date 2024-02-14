Feature: Any
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Use an any type
    Given a file named "Foo.pen" with:
      """pen
      f = \() any {
        42
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Downcast an any type
    Given a file named "Foo.pen" with:
      """pen
      f = \(x any) number {
        if x = x as number {
          x
        } else {
          0
        }
      }
      """
    When I run `pen build`
    Then the exit status should be 0
