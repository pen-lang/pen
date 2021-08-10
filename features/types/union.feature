Feature: Union
  Background:
    Given a file named "pen.json" with:
    """json
    { "dependencies": {} }
    """

  Scenario: Upcast a number into a union
    Given a file named "Foo.pen" with:
    """pen
    f = \() number | none {
      42
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Upcast a number list into a union
    Given a file named "Foo.pen" with:
    """pen
    f = \() [number] | none {
      [number; 42]
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Downcast a union type
    Given a file named "Foo.pen" with:
    """pen
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
    """pen
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

  Scenario: Downcast a union type to another union type
    Given a file named "Foo.pen" with:
    """pen
    f = \(x number | boolean | none) number | none {
      if x = x; number | none {
        x
      } else {
        none
      }
    }
    """
    When I run `pen build`
    Then the exit status should be 0
