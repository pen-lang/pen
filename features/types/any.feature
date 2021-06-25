Feature: Any
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Use an any type
    Given a file named "Foo.pen" with:
    """
    f = \() any {
      42
    }
    """
    When I run `pen build`
    Then the exit status should be 0
