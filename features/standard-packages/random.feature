Feature: Random
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Core": "pen:///core",
          "Os": "pen:///os",
          "Random": "pen:///random"
        }
      }
      """

  Scenario: Generate a random number
    Given a file named "main.pen" with:
      """pen
      import Core'Number
      import Os'Context { Context }
      import Os'File
      import Os'Process
      import Random'Random

      main = \(ctx context) none {
        if m = run(ctx) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx context) none | error {
        File'Write(ctx.Os, File'StdOut(), Number'String(Random'Number(ctx.Random)))?

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
