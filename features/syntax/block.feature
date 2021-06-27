Feature: Block
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Define a variable
    Given a file named "Foo.pen" with:
    """
    f = \(x number) number {
      y = x

      y
    }
    """
    When I run `pen build`
    Then the exit status should be 0
