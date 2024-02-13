Feature: List as stream
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
    And a file named "Hello.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File

      Hello = \(ctx Context) none {
        _ = File'Write(ctx, File'StdOut(), "hello")

        none
      }
      """

  Scenario: Evaluate an element lazily
    Given a file named "main.pen" with:
      """pen
      import 'Hello

      main = \(ctx context) none {
        [none Hello'Hello(ctx.Os)]

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should not contain "hello"

  Scenario: Evaluate an element lazily but only once
    Given a file named "main.pen" with:
      """pen
      import Os'Process
      import 'Hello

      main = \(ctx context) none {
        if [x, ...xs] = [none Hello'Hello(ctx.Os)] {
          x()
          x()

          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "hello"

  Scenario: Evaluate multiple elements lazily
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import 'Hello

      foo = \(ctx Context) [none] {
        Hello'Hello(ctx)

        [none]
      }

      main = \(ctx context) none {
        [none ...foo(ctx.Os)]

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should not contain "hello"

  Scenario: Evaluate multiple elements lazily but only once
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'Process
      import 'Hello

      foo = \(ctx Context) [none] {
        Hello'Hello(ctx)

        [none]
      }

      main = \(ctx context) none {
        xs = [none ...foo(ctx.Os)]

        if [x, ...xs] = xs {
          Process'Exit(ctx.Os, 1)
        } else {
          if [x, ...xs] = xs {
            Process'Exit(ctx.Os, 1)
          } else {
            none
          }
        }
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "hello"
