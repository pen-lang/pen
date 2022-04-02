Feature: Formatting module files
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "library",
      "dependencies": {}
    }
    """

  Scenario: Format a package
    Given a file named "Foo.pen" with:
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

  Scenario: Format a package
    Given a file named "Foo.pen" with:
    """pen
    Foo = \() none {

      none
    }
    """
    When I run `pen format --checked`
    Then the exit status should not be 0
