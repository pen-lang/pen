Feature: Record
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Create a record with an element
    Given a file named "Foo.pen" with:
    """
    type r {
      x number,
    }

    f = \() r {
      r{x: 42}
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Create a record with two elements
    Given a file named "Foo.pen" with:
    """
    type r {
      x number,
      y none,
    }

    f = \() r {
      r{x: 42, y: none}
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Create a record without no element
    Given a file named "Foo.pen" with:
    """
    type r {}

    f = \() r {
      r{}
    }
    """
    When I run `pen build`
    Then the exit status should be 0
