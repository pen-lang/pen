Feature: Map
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "library",
      "dependencies": {}
    }
    """

  Scenario: Create an empty map
    Given a file named "Foo.pen" with:
    """pen
    f = \() {string:number} {
      {string:number}
    }
    """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Create a map with an entry
    Given a file named "Foo.pen" with:
    """pen
    f = \() {string:number} {
      {string:number "foo": 42}
    }
    """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Create a map with two elements
    Given a file named "Foo.pen" with:
    """pen
    f = \() {string:number} {
      {string:number "foo": 1, "bar": 2}
    }
    """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Merge maps
    Given a file named "Foo.pen" with:
    """pen
    f = \(xs {string:number}) {string:number} {
      {string:number ...xs, ...xs}
    }
    """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Create a map of a union type key
    Given a file named "Foo.pen" with:
    """pen
    f = \() {string|none:number} {
      {string|none:number "foo": 1, none: 2}
    }
    """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Create a map of a union type value
    Given a file named "Foo.pen" with:
    """pen
    f = \() {string:number|none} {
      {string:number|none "foo": 42, "bar": none}
    }
    """
    When I successfully run `pen build`
    Then the exit status should be 0
