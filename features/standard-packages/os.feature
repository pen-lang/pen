Feature: OS
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///lib/os",
        "Core": "pen:///lib/core"
      }
    }
    """

  Scenario: Get arguments
    Given a file named "Main.pen" with:
    """pen
    import Core'String
    import System'Os

    main = \(ctx Os'Context) number {
      if _ = Os'WriteFile(ctx, Os'StdOut(), String'Join(Os'Arguments(ctx), " ")) as number {
        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app foo bar`
    And stdout from "./app foo bar" should contain exactly "foo bar"

  Scenario: Get an environment variable
    Given a file named "Main.pen" with:
    """pen
    import Core'String
    import System'Os

    printEnvironmentVariable = \(ctx Os'Context) none | error {
      Os'WriteFile(ctx, Os'StdOut(), Os'EnvironmentVariable(ctx, "FOO")?)?

      none
    }

    main = \(ctx Os'Context) number {
      if _ = printEnvironmentVariable(ctx) as none {
        0
      } else {
        1
      }
    }
    """
    And I append "foo" to the environment variable "FOO"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And stdout from "./app" should contain exactly "foo"

  Scenario: Open a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(ctx Os'Context) number {
      if f = Os'OpenFile(ctx, "./foo.txt") as Os'File {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Read a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    readFile = \(ctx Os'Context) none | error {
      f = Os'OpenFile(ctx, "foo.txt")?
      d = Os'ReadFile(ctx, f)?
      f = Os'OpenFileWithOptions(
        ctx,
        "bar.txt",
        Os'OpenFileOptions{
          ...Os'DefaultOpenFileOptions(),
          Create: true,
          Write: true,
        },
      )?
      Os'WriteFile(ctx, f, d)?

      none
    }

    main = \(ctx Os'Context) number {
      if _ = readFile(ctx) as none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "bar.txt" should contain "foo"

  Scenario: Write a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    writeFile = \(ctx Os'Context) none | error {
      f = Os'OpenFileWithOptions(
        ctx,
        "./foo.txt",
        Os'OpenFileOptions{...Os'DefaultOpenFileOptions(), Write: true},
      )?

      Os'WriteFile(ctx, f, "foo")?

      none
    }

    main = \(ctx Os'Context) number {
      if _ = writeFile(ctx) as none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "foo.txt" should contain "foo"

  Scenario: Copy a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(ctx Os'Context) number {
      if _ = Os'CopyFile(ctx, "foo.txt", "bar.txt") as none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "bar.txt" should contain "foo"

  Scenario: Remove a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(ctx Os'Context) number {
      if _ = Os'RemoveFile(ctx, "foo.txt") as none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "foo.txt" does not exist

  Scenario: Read a directory
    Given a file named "Main.pen" with:
    """pen
    import System'Os
    import Core'String

    readDirectory = \(ctx Os'Context) none | error {
      Os'WriteFile(
        ctx,
        Os'StdOut(),
        String'Join(Os'ReadDirectory(ctx, ".")?, "\n"),
      )?

      none
    }

    main = \(ctx Os'Context) number {
      if _ = readDirectory(ctx) as none {
        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain "Main.pen"
    And the stdout from "./app" should contain "pen.json"

  Scenario: Create a directory
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(ctx Os'Context) number {
      if _ = Os'CreateDirectory(ctx, "foo") as none {
        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And a directory named "foo" should exist

  Scenario: Remove a directory
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(ctx Os'Context) number {
      if _ = Os'RemoveDirectory(ctx, "foo") as none {
        0
      } else {
        1
      }
    }
    """
    And a directory named "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And a directory named "foo" should not exist
