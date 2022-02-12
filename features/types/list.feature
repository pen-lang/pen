Feature: List
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "library",
      "dependencies": {}
    }
    """

  Scenario: Create an empty list
    Given a file named "Foo.pen" with:
    """pen
    f = \() [number] {
      [number]
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Create a list with an element
    Given a file named "Foo.pen" with:
    """pen
    f = \() [number] {
      [number 1]
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Create a list with two elements
    Given a file named "Foo.pen" with:
    """pen
    f = \() [number] {
      [number 1, 2]
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Join lists
    Given a file named "Foo.pen" with:
    """pen
    f = \(xs [number]) [number] {
      [number ...xs, ...xs]
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Create a list of a union type
    Given a file named "Foo.pen" with:
    """pen
    f = \() [number | none] {
      [number | none 1, none]
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Coerce elements of a spread list
    Given a file named "Foo.pen" with:
    """pen
    f = \(xs [number]) [number | none] {
      [number | none ...xs]
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use if-list expression
    Given a file named "Foo.pen" with:
    """pen
    f = \(xs [number]) [number] {
      if [y, ...ys] = xs {
        [number y(), ...ys]
      } else {
        [number]
      }
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Use list comprehension
    Given a file named "Foo.pen" with:
    """pen
    f = \(xs [number]) [number] {
      [number x() + 42 for x in xs]
    }
    """
    When I run `pen build`
    Then the exit status should be 0
