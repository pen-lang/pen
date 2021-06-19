Feature: Module compiler
  Background:
    Given a file named "pen.json" with:
    """
    {}
    """

  Scenario: Compile a module
    Given a file named "Foo.pen" with:
    """
    f = \(x number) number {
      x
    }
    """
    When I run `pen compile -p foo. -m foo. Foo.pen Foo.bc Foo.json`
    Then the exit status should be 0
