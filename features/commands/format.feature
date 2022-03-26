Feature: Formatting module files
  Scenario: Format a package
    Given a file named "pen.json" with:
    """json
    {
      "type": "library",
      "dependencies": {}
    }
    """
    And a file named "Foo.pen" with:
    """pen
    Foo = \() none {

      none
    }
    """
    When I successfully run `pen format`
    Then a file named "Foo.pen" should contain exactly:
    """
    Foo = \() none {
      none
    }
    """
