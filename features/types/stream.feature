Feature: List as stream
  Background:
    Given a file named "pen.json" with:
    """json
    {
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
      File'Write(ctx, File'StdOut(), "hello")

      none
    }
    """

  Scenario: Evaluate an element lazily
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }
    import 'Hello

    main = \(ctx context) none {
      [none Hello'Hello(ctx)]

      none
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should not contain "hello"

  Scenario: Evaluate an element lazily but only once
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }
    import 'Hello

    main = \(ctx context) none {
      if [x, ...xs] = [none Hello'Hello(ctx)] {
        x()
        x()

        none
      } else {
        none
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
      [none ...foo(ctx)]

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
    import 'Hello

    foo = \(ctx Context) [none] {
      Hello'Hello(ctx)

      [none]
    }

    main = \(ctx context) none {
      if [x, ...xs] = [none ...foo(ctx)] {
        x()
        x()

        none
      } else {
        none
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "hello"
