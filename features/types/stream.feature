Feature: List as stream
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///lib/os"
      }
    }
    """
    And a file named "Hello.pen" with:
    """pen
    import System'Os

    Hello = \(ctx Os'Context) none {
      Os'WriteFile(ctx, Os'StdOut(), "hello")

      none
    }
    """

  Scenario: Evaluate an element lazily
    Given a file named "Main.pen" with:
    """pen
    import System'Os
    import 'Hello

    main = \(ctx Os'Context) number {
      [none; Hello'Hello(ctx)]

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should not contain "hello"

  Scenario: Evaluate an element lazily but only once
    Given a file named "Main.pen" with:
    """pen
    import System'Os
    import 'Hello

    main = \(ctx Os'Context) number {
      if [x, ...xs] = [none; Hello'Hello(ctx)] {
        x()
        x()
      } else {
        none
      }

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "hello"
