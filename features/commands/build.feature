Feature: Building packages
  Scenario: Build an application package
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///os"
      }
    }
    """
    And a file named "main.pen" with:
    """pen
    import System'Context { Context }

    main = \(ctx Context) number {
      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Build a library package
    Given a file named "pen.json" with:
    """json
    { "dependencies": {} }
    """
    And a file named "Foo.pen" with:
    """pen
    f = \(x number) number {
      x
    }
    """
    When I run `pen build`
    Then the exit status should be 0

  Scenario: Cross-build an application package
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///os"
      }
    }
    """
    And a file named "main.pen" with:
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
      | wasm32-wasi                |

  Scenario Outline: Cross-build a library package
    Given a file named "pen.json" with:
    """json
    { "dependencies": {} }
    """
    And a file named "Foo.pen" with:
    """pen
    f = \(x number) number {
      x
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
      | wasm32-wasi                |
