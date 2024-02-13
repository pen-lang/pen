Feature: FFI
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Import a foreign function of native calling convention
    Given a file named "Foo.pen" with:
      """pen
      import foreign g \(number) number

      f = \(x number) number {
        g(x)
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Import a foreign function of the C calling convention
    Given a file named "Foo.pen" with:
      """pen
      import foreign "c" g \(number) number

      f = \(x number) number {
        g(x)
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Export a foreign function
    Given a file named "Foo.pen" with:
      """pen
      foreign f = \(x number) number {
        x
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Export a foreign function of the C calling convention
    Given a file named "Foo.pen" with:
      """pen
      foreign "c" f = \(x number) number {
        x
      }
      """
    When I run `pen build`
    Then the exit status should be 0
