Feature: Package builder
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Build a package
    Given a file named "Foo.pen" with:
    """
    f = \(x number) number {
      x
    }
    """
    When I run `pen build`
    Then the exit status should be 0
