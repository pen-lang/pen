Feature: OS (asynchronous runtime)
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///os-async",
        "Core": "pen:///core"
      }
    }
    """

  Scenario: Cross-build an application package
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    main = \(ctx Context) number {
      0
    }
    """
    When I run `pen build --target <target>`
    Then the exit status should be 0

    Examples:
      | target                     |
      | i686-unknown-linux-musl    |
      | x86_64-unknown-linux-musl  |
      | aarch64-unknown-linux-musl |

  Scenario: Run an application
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }
    import System'File

    main = \(ctx Context) number {
      if _ = run(ctx) as none {
        0
      } else {
        1
      }
    }

    run = \(ctx Context) none | error {
      File'Write(ctx, File'StdOut(), "Hello, world!")?

      none
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "Hello, world!"
