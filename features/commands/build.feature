Feature: Building packages
  Scenario: Build an application package
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Os": "pen:///os"
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
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Build a library package
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """
    And a file named "Foo.pen" with:
      """pen
      f = \(x number) number {
        x
      }
      """
    When I run `pen build`
    Then the exit status should be 0

  Scenario Outline: Cross-build an application package
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Os": "pen:///os-sync"
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
    And I successfully run `rustup target add <target>`
    When I successfully run `pen build --target <target>`
    Then the exit status should be 0

    Examples:
      | target                     |
      | i686-unknown-linux-musl    |
      | x86_64-unknown-linux-musl  |
      | aarch64-unknown-linux-musl |
      | wasm32-wasi                |

  Scenario Outline: Cross-build a library package
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """
    And a file named "Foo.pen" with:
      """pen
      f = \(x number) number {
        x
      }
      """
    And I successfully run `rustup target add <target>`
    When I successfully run `pen build --target <target>`
    Then the exit status should be 0

    Examples:
      | target                     |
      | i686-unknown-linux-musl    |
      | x86_64-unknown-linux-musl  |
      | aarch64-unknown-linux-musl |
      | wasm32-wasi                |

  Scenario: Build an application package again
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Os": "pen:///os"
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
    And I successfully run `pen build`
    And I successfully run `./app`
    And the stdout from "./app" should contain exactly ""
    When a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File

      main = \(ctx context) none {
        _ = File'Write(ctx.Os, File'StdOut(), "hello")

        none
      }
      """
    And I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "hello"
