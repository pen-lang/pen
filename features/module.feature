Feature: Module
  Background:
    Given a file named "pen.json" with:
    """
    { "dependencies": {} }
    """

  Scenario: Import a function from a module
    Given a file named "Foo.pen" with:
    """
    Foo = \() number {
      42
    }
    """
    And a file named "Bar.pen" with:
    """
    import 'Foo

    Bar = \() number {
      Foo'Foo()
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Import a type alias from a module
    Given a file named "Foo.pen" with:
    """
    type Foo = number
    """
    And a file named "Bar.pen" with:
    """
    import 'Foo

    type Bar = Foo'Foo
    """
    When I run `pen build`
    Then the exit status should be 0
