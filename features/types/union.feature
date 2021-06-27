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

  Scenario: Downcast a union type
    Given a file named "Foo.pen" with:
    """
    f = \(x number | none) number {
      if x = x; number {
        x
      } else if none {
        0
      }
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Downcast a union type with an else block
    Given a file named "Foo.pen" with:
    """
    f = \(x number | none) number {
      if x = x; none {
        0
      } else {
        x
      }
    }
    """
    When I run `pen build`
    Then the exit status should be 0
