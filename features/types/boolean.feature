Feature: Boolean
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Use boolean literals
    Given a file named "Foo.pen" with:
    """
    f = \() boolean {
      true
    }

    g = \() boolean {
      false
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use an and operation
    Given a file named "Foo.pen" with:
    """
    f = \() boolean {
      true & false
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use an or operation
    Given a file named "Foo.pen" with:
    """
    f = \() boolean {
      true | false
    }
    """
    When I run `pen build`
    Then the exit status should be 0
