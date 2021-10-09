Feature: Record
  Background:
    Given a file named "pen.json" with:
    """json
    { "dependencies": {} }
    """

  Scenario: Create a record with an field
    Given a file named "Foo.pen" with:
    """pen
    type r {
      x number
    }

    f = \() r {
      r{x: 42}
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Create a record with two fields
    Given a file named "Foo.pen" with:
    """pen
    type r {
      x number
      y none
    }

    f = \() r {
      r{x: 42, y: none}
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Create a record with no field
    Given a file named "Foo.pen" with:
    """pen
    type r {}

    f = \() r {
      r
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Update a record
    Given a file named "Foo.pen" with:
    """pen
    type r {
      x number
      y none
    }

    f = \(x r) r {
      r{...x, y: none}
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Get an field in a record
    Given a file named "Foo.pen" with:
    """pen
    type r {
      x number
    }

    f = \(x r) number {
      x.x
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use an equal operator
    Given a file named "Foo.pen" with:
    """pen
    type r {
      x number
    }

    f = \(x r, y r) boolean {
      x == y
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use a not-equal operator
    Given a file named "Foo.pen" with:
    """pen
    type r {
      x number
    }

    f = \(x r, y r) boolean {
      x == y
    }
    """
    When I run `pen build`
    Then the exit status should be 0
