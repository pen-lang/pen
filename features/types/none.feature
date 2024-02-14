Feature: None
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Use a none literal
    Given a file named "Foo.pen" with:
      """pen
      f = \() none {
        none
      }
      """
    When I run `pen build`
    Then the exit status should be 0
