Feature: Block
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Define a variable
    Given a file named "Foo.pen" with:
      """pen
      f = \(x number) number {
        y = x

        y
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Call a function
    Given a file named "Foo.pen" with:
      """pen
      f = \() none {
        none
      }

      g = \() none {
        f()

        none
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use if expression
    Given a file named "Foo.pen" with:
      """pen
      f = \() none {
        none
      }

      g = \() none {
        none
      }

      h = \(x boolean) none {
        if x {
          f()
        } else {
          g()
        }

        none
      }
      """
    When I run `pen build`
    Then the exit status should be 0
