Feature: Record
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "library",
      "dependencies": {}
    }
    """

  Scenario: Import a record with no field
    Given a file named "foo.pen" with:
    """pen
    import 'bar

    x = \() bar'Bar { bar'Bar{} }
    """
    And a file named "bar.pen" with:
    """pen
    type Bar {}
    """
    When I successfully run `pen build`
    Then the exit status should be 0

  Scenario: Import a record with no field without a module prefix
    Given a file named "foo.pen" with:
    """pen
    import 'bar { Bar }

    x = \() Bar { Bar{} }
    """
    And a file named "bar.pen" with:
    """pen
    type Bar {}
    """
    When I successfully run `pen build`
    Then the exit status should be 0
