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
    import System'Context { Context }
    import System'Environment
    import System'File

    main = \(ctx Context) number {
      if _ = File'Write(ctx, File'StdOut(), String'Join(Environment'Arguments(ctx), " ")) as number {
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
    import System'Context { Context }
    import System'File
    import System'Environment

    printEnvironmentVariable = \(ctx Context) none | error {
      File'Write(ctx, File'StdOut(), Environment'Variable(ctx, "FOO")?)?

      none
    }

    main = \(ctx Context) number {
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
    import System'Context { Context }
    import System'File { File }

    main = \(ctx Context) number {
      if f = File'Open(ctx, "./foo.txt") as File {
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
    import System'Context { Context }
    import System'File
    import System'File'OpenOptions

    readFile = \(ctx Context) none | error {
      f = File'Open(ctx, "foo.txt")?
      d = File'Read(ctx, f)?
      f = File'OpenWithOptions(
        ctx,
        "bar.txt",
        OpenOptions{
          ...OpenOptions'Default(),
          Create: true,
          Write: true,
        },
      )?
      File'Write(ctx, f, d)?

      none
    }

    main = \(ctx Context) number {
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
    import System'Context { Context }
    import System'File
    import System'File'OpenOptions

    writeFile = \(ctx Context) none | error {
      f = File'OpenWithOptions(
        ctx,
        "./foo.txt",
        OpenOptions{...OpenOptions'Default(), Write: true},
      )?

      File'Write(ctx, f, "foo")?

      none
    }

    main = \(ctx Context) number {
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
    import System'Context { Context }
    import System'File

    main = \(ctx Context) number {
      if _ = File'Copy(ctx, "foo.txt", "bar.txt") as none {
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
    import System'Context { Context }
    import System'File

    main = \(ctx Context) number {
      if _ = File'Remove(ctx, "foo.txt") as none {
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
    import Core'String
    import System'Context { Context }
    import System'File
    import System'Directory

    readDirectory = \(ctx Context) none | error {
      File'Write(
        ctx,
        File'StdOut(),
        String'Join(Directory'Read(ctx, ".")?, "\n"),
      )?

      none
    }

    main = \(ctx Context) number {
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
    import System'Context { Context }
    import System'Directory

    main = \(ctx Context) number {
      if _ = Directory'Create(ctx, "foo") as none {
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
    import System'Context { Context }
    import System'Directory

    main = \(ctx Context) number {
      if _ = Directory'Remove(ctx, "foo") as none {
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
