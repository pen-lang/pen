Feature: Function
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Define a function
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number) number {
        x
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Call a function with no argument
    Given a file named "Foo.pen" with:
      """pen
      f = \() number {
        f()
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Call a function with an argument
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number) number {
        f(x)
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Call a function with two arguments
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number, y number) number {
        f(x, y)
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Define a closure
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number) \(number) number {
        \(y number) number {
          x + y
        }
      }
      """
    When I run `pen build`
    Then the exit status should be 0
