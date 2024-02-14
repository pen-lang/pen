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
      type foo {}

      type bar {
        x number
      }

      type baz {
        x number
        y string
      }

      main = \(_ context) none {
        debug(<input>)
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stderr from "./app" should contain exactly '<output>'
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | input                               | output                              |
      | false                               | false                               |
      | true                                | true                                |
      | none                                | none                                |
      | "foo"                               | "foo"                               |
      | 42                                  | 42                                  |
      | foo{}                               | foo{}                               |
      | bar{x: 42}                          | bar{x: 42}                          |
      | baz{x: 42, y: "foo"}                | baz{x: 42, y: "foo"}                |
      | [number 42, 42]                     | [number 42, 42]                     |
      | {string: number "foo": 1, "bar": 2} | {string: number "bar": 2, "foo": 1} |
      | error("foo")                        | error("foo")                        |
      | \\(x none) none { x }               | <function>                          |
