Feature: Modules
  Background:
    Given a file named "pen.json" with:
    """json
    { "dependencies": {} }
    """

  Scenario: Import a function from a module
    Given a file named "Foo.pen" with:
    """pen
    Foo = \() number {
      42
    }
    """
    And a file named "Bar.pen" with:
    """pen
    import 'Foo

    Bar = \() number {
      Foo'Foo()
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Import a type alias from a module
    Given a file named "Foo.pen" with:
    """pen
    type Foo = number
    """
    And a file named "Bar.pen" with:
    """pen
    import 'Foo

    type Bar = Foo'Foo
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Import a function from a nested module
    Given a file named "Foo/Foo.pen" with:
    """pen
    Foo = \() number {
      42
    }
    """
    And a file named "Bar.pen" with:
    """pen
    import 'Foo'Foo

    Bar = \() number {
      Foo'Foo()
    }
    """
    When I run `pen build`
    Then the exit status should be 0
