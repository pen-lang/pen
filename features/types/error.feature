Feature: Error
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Call a error function
    Given a file named "Foo.pen" with:
    """
    f = \() error {
      error(none)
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Call a source function
    Given a file named "Foo.pen" with:
    """
    f = \(e error) any {
      source(e)
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use a try operator
    Given a file named "Foo.pen" with:
    """
    f = \(x number | error) number | error {
      x? + 1
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use a try operator with a union type
    Given a file named "Foo.pen" with:
    """
    f = \(x number | none | error) number | error {
      if x = x?; number {
        x + 1
      } else if none {
        0
      }
    }
    """
    When I run `pen build`
    Then the exit status should be 0
