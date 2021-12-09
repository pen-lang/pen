Feature: OS (synchronous version)
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///os-sync",
        "Core": "pen:///core"
      }
    }
    """

  Scenario: Build an application
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

  Scenario: Get arguments
    Given a file named "main.pen" with:
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
    Given a file named "main.pen" with:
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
    Given a file named "main.pen" with:
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
    Given a file named "main.pen" with:
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

  Scenario: Read a file until a limit
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }
    import System'File

    readFile = \(ctx Context) none | error {
      f = File'Open(ctx, "foo.txt")?
      d = File'ReadLimit(ctx, f, 5)?
      File'Write(ctx, File'StdOut(), d)?

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
    And a file named "foo.txt" with "Hello, world!"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "Hello"

  Scenario: Write a file
    Given a file named "main.pen" with:
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
    Given a file named "main.pen" with:
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
    And the file "foo.txt" should contain "foo"
    And the file "bar.txt" should contain "foo"

  Scenario: Move a file
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }
    import System'File

    main = \(ctx Context) number {
      if _ = File'Move(ctx, "foo.txt", "bar.txt") as none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "foo.txt" does not exist
    And the file "bar.txt" should contain "foo"

  Scenario: Remove a file
    Given a file named "main.pen" with:
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
    Given a file named "main.pen" with:
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
    And the stdout from "./app" should contain "main.pen"
    And the stdout from "./app" should contain "pen.json"

  Scenario: Create a directory
    Given a file named "main.pen" with:
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
    Given a file named "main.pen" with:
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

  Scenario: Get file metadata
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }
    import System'File
    import System'File'Metadata { Metadata }

    main = \(ctx Context) number {
      m = File'Metadata(ctx, "foo")

      if m = m as Metadata {
        if m.Size == 3 {
          0
        } else {
          1
        }
      } else {
        1
      }
    }
    """
    And a file named "foo" with:
    """
    foo
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Get system time
    Given a file named "main.pen" with:
    """pen
    import Core'Number
    import System'Context { Context }
    import System'File
    import System'Time

    run = \(ctx Context) none | error {
      File'Write(ctx, File'StdOut(), Number'String(Time'Now(ctx)))?

      none
    }

    main = \(ctx Context) number {
      if m = run(ctx) as none {
        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Sleep
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }
    import System'Time

    main = \(ctx Context) number {
      Time'Sleep(ctx, 1)

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Generate a random number
    Given a file named "main.pen" with:
    """pen
    import Core'Number
    import System'Context { Context }
    import System'File
    import System'Random

    run = \(ctx Context) none | error {
      File'Write(ctx, File'StdOut(), Number'String(Random'Number(ctx)))?

      none
    }

    main = \(ctx Context) number {
      if m = run(ctx) as none {
        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
