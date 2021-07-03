Feature: Polymorphism
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Use an equal operator
    Given a file named "Foo.pen" with:
    """
    f = \() boolean {
      0 == 0
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use a not-equal operator
    Given a file named "Foo.pen" with:
    """
    f = \() boolean {
      0 != 0
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Compare unions
    Given a file named "Foo.pen" with:
    """
    f = \(x number | none, y number | none) boolean {
      x == y
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use a union and none
    Given a file named "Foo.pen" with:
    """
    f = \(x number | none) boolean {
      x == none
    }
    """
    When I run `pen build`
    Then the exit status should be 0
