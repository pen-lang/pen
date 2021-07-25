Feature: Building packages
  Scenario: Build an application package
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "file://pen-root/lib/os"
      }
    }
    """
    And a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(os Os'Os) number {
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
        "System": "file://pen-root/lib/os"
      }
    }
    """
    And a file named "Foo.pen" with:
    """pen
    import System'Os

    main = \(os Os'Os) number {
      0
    }
    """
    # TODO Use wasm32-unknown-unknown fixing the os standard package.
    When I run `pen build --target x86_64-unknown-linux-musl`
    Then the exit status should be 0

  Scenario: Cross-build a library package
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
    When I run `pen build --target wasm32-unknown-unknown`
    Then the exit status should be 0
