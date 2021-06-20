Feature: Function call
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Call a function
    Given a file named "Foo.pen" with:
    """
    f = \(x number) number {
      x
    }

    g = \(x number) number {
      f(x)
    }
    """
    When I run `pen build`
    Then the exit status should be 0
