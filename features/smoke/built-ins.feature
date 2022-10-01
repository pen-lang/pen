Feature: Built-ins
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "application",
      "dependencies": {
        "Os": "pen:///os"
      }
    }
    """

  Scenario Outline: Print a value with a debug function
    Given a file named "main.pen" with:
    """pen
    main = \(_ context) none {
      debug(<input>)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stderr from "./app" should contain exactly "<output>"
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | input | output  |
      | false | false   |
      | true  | true    |
      | none  | none    |
      | "foo" | \"foo\" |
      | 42    | 42      |
