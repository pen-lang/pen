Feature: Modules
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "library",
      "dependencies": {}
    }
    """

  Scenario: Compare a type imported indirectly
    Given a file named "Foo.pen" with:
    """pen
    type Foo {}
    """
    And a file named "Bar.pen" with:
    """pen
    import 'foo { Foo }

    type Bar {
      foo Foo
    }
    """
    And a file named "Baz.pen" with:
    """pen
    import 'bar { Bar }

    compare = \(x Bar, y Bar) boolean {
      x == y
    }
    """
    When I run `pen build`
    Then the exit status should be 0
