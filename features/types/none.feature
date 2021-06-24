Feature: None
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Use a none literal
    Given a file named "Foo.pen" with:
    """
    f = \() none {
      none
    }
    """
    When I run `pen build`
    Then the exit status should be 0
