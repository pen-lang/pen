Feature: Module compiler
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Compile a module
    Given a file named "Foo.pen" with:
    """
    f = \(x number) number {
      x
    }
    """
    When I run `pen compile Foo.pen Foo.bc Foo.json`
    Then the exit status should be 0
