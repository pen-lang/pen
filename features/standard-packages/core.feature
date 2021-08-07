Feature: Core
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "Core": "file://pen-root/lib/core"
      }
    }
    """

  Scenario: Convert a number to a string
    Given a file named "Foo.pen" with:
    """pen
    import Core'Number

    f = \() string {
      Number'String(42)
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Join strings
    Given a file named "Foo.pen" with:
    """pen
    import Core'String

    f = \() string {
      String'Join([string; "hello", "world"], " ")
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Slice a string
    Given a file named "Foo.pen" with:
    """pen
    import Core'String

    f = \() string {
      String'Slice("foo", 1, 2)
    }
    """
    When I run `pen build`
    Then the exit status should be 0
