Feature: Testing
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {
          "Test": "pen:///test"
        }
      }
      """
    And a file named "Foo.pen" with:
      """pen
      Add = \(x number, y number) number {
        x + y
      }
      """

  Scenario: Run a test referencing an Os package
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Os": "pen:///os",
          "Test": "pen:///test"
        }
      }
      """
    And a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      main = \(ctx context) none {
        none
      }
      """
    And a file named "main.test.pen" with:
      """pen
      import Os'File

      Foo = \() none | error {
        _ = File'Write

        none
      }
      """
    When I successfully run `pen test`
    Then the exit status should be 0

  Scenario: Run tests without a Test package
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """
    And a file named "Foo.test.pen" with:
      """pen
      import 'Foo

      Add = \() none | error {
        if Foo'Add(41, 1) == 42 { none } else { error("oh no") }
      }
      """
    When I successfully run `pen test`
    Then the exit status should be 0

  Scenario: Keep an application file on testing
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Os": "pen:///os",
          "Test": "pen:///test"
        }
      }
      """
    And a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      main = \(ctx context) none {
        none
      }
      """
    And a file named "main.test.pen" with:
      """pen
      Foo = \() none | error {
        none
      }
      """
    And I successfully run `pen build`
    When I successfully run `pen test`
    Then I successfully run `./app`
