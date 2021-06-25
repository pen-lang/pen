Feature: Union
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Use a union type
    Given a file named "Foo.pen" with:
    """
    f = \() number | none {
      42
    }
    """
    When I run `pen build`
    Then the exit status should be 0
