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
