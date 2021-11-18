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
    And I successfully run `rustup target add <target>`
    When I run `pen build --target <target>`
    Then the exit status should be 0

    Examples:
      | target                     |
      | i686-unknown-linux-musl    |
      | x86_64-unknown-linux-musl  |
      | aarch64-unknown-linux-musl |
