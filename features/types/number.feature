Feature: Number
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Use a number literal
    Given a file named "Foo.pen" with:
    """
    f = \() number {
      42
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use arithmetic operators
    Given a file named "Foo.pen" with:
    """
    f = \() number {
      1 + 2 - 3 * 4 / 5
    }
    """
    When I run `pen build`
    Then the exit status should be 0
