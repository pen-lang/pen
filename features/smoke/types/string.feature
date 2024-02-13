Feature: String
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

  Scenario: Concatenate zero-length strings
    Given a file named "main.pen" with:
      """pen
      import Os'Process

      main = \(ctx context) none {
        debug("" + "foo" + "")
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
    And I successfully run `./app`
    And the stderr from "./app" should contain exactly "\"foo\""
