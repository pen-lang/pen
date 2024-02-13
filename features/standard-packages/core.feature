Feature: Core
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {
          "Core": "pen:///core"
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

  Scenario: Sum numbers
    Given a file named "Foo.pen" with:
      """pen
      import Core'Number

      f = \() number {
        Number'Sum([number 1, 2, 3])
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Join strings
    Given a file named "Foo.pen" with:
      """pen
      import Core'String

      f = \() string {
        String'Join([string "hello", "world"], " ")
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Slice a string as bytes
    Given a file named "Foo.pen" with:
      """pen
      import Core'String'Byte

      f = \() string {
        Byte'Slice("foo", 1, 2)
      }
      """
    When I run `pen build`
    Then the exit status should be 0
