Feature: Generating documentation for a package
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "library",
      "dependencies": {}
    }
    """

  Scenario: Generate documentation for a package
    Given a file named "Foo.pen" with:
    """pen
    # Do something nice.
    Foo = \() none {
      none
    }
    """
    When I run the following script:
    """sh
    pen document Foo > Foo.md
    """
    Then a file named "Foo.md" should contain "`Foo` package"
