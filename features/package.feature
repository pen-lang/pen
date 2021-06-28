Feature: Package
  Background:
    Given a file named "foo/pen.json" with:
    """
    { "dependencies": {} }
    """
    And a file named "foo/Foo.pen" with:
    """
    Foo = \() number {
      42
    }
    """
    And a directory named "bar"
    And I cd to "bar"
    And a file named "pen.json" with:
    """
    {
      "dependencies": {
        "Foo": "file+relative:../foo"
      }
    }
    """

  Scenario: Import a function from a module
    Given a file named "Bar.pen" with:
    """
    import Foo'Foo

    Bar = \() number {
      Foo'Foo()
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Import a type alias from a module
    Given a file named "Bar.pen" with:
    """
    import Foo'Foo

    type Bar = Foo'Foo
    """
    When I run `pen build`
    Then the exit status should be 0
