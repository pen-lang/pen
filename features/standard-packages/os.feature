Feature: Os
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Os": "pen:///os",
          "Core": "pen:///core"
        }
      }
      """

  Scenario: Build an application
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
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
      import Os'Context { Context }
      import Os'Environment
      import Os'File
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        File'Write(ctx, File'StdOut(), String'Join(Environment'Arguments(ctx), " "))?

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app foo bar`
    And stdout from "./app foo bar" should contain exactly "foo bar"

  Scenario: Get an environment variable
    Given a file named "main.pen" with:
      """pen
      import Core'String
      import Os'Context { Context }
      import Os'File
      import Os'Environment
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        File'Write(ctx, File'StdOut(), Environment'Variable(ctx, "FOO")?)?

        none
      }
      """
    And I append "foo" to the environment variable "FOO"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And stdout from "./app" should contain exactly "foo"

  Scenario: Open a file
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File { File }
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        File'Open(ctx, "./foo.txt")?

        none
      }
      """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Read a file
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File
      import Os'File'OpenOptions
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        f = File'Open(ctx, "foo.txt")?
        d = File'Read(ctx, f)?

        File'Write(ctx, File'StdOut(), d)?

        none
      }
      """
    And a file named "foo.txt" with "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "foo"

  Scenario: Read a file until a limit
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        f = File'Open(ctx, "foo.txt")?
        d = File'ReadLimit(ctx, f, 5)?
        File'Write(ctx, File'StdOut(), d)?

        none
      }
      """
    And a file named "foo.txt" with "Hello, world!"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain exactly "Hello"

  Scenario: Write a file
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File
      import Os'File'OpenOptions { OpenOptions }
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        f = File'OpenWithOptions(
          ctx,
          "./foo.txt",
          OpenOptions{...OpenOptions'Default(), Write: true},
        )?

        File'Write(ctx, f, "foo")?

        none
      }
      """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "foo.txt" should contain "foo"

  Scenario: Copy a file
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File
      import Os'Process

      main = \(ctx context) none {
        if _ = File'Copy(ctx.Os, "foo.txt", "bar.txt") as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
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
      import Os'Context { Context }
      import Os'File
      import Os'Process

      main = \(ctx context) none {
        if _ = File'Move(ctx.Os, "foo.txt", "bar.txt") as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
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
      import Os'Context { Context }
      import Os'File
      import Os'Process

      main = \(ctx context) none {
        if _ = File'Remove(ctx.Os, "foo.txt") as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
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
      import Os'Context { Context }
      import Os'File
      import Os'Directory
      import Os'Process

      main = \(ctx context) none {
        if _ = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        File'Write(
          ctx,
          File'StdOut(),
          String'Join(Directory'Read(ctx, ".")?, "\n"),
        )?

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the stdout from "./app" should contain "main.pen"
    And the stdout from "./app" should contain "pen.json"

  Scenario: Create a directory
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'Directory
      import Os'Process

      main = \(ctx context) none {
        if _ = Directory'Create(ctx.Os, "foo") as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`
    And a directory named "foo" should exist

  Scenario: Remove a directory
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'Directory
      import Os'Process

      main = \(ctx context) none {
        if _ = Directory'Remove(ctx.Os, "foo") as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
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
      import Os'Context { Context }
      import Os'File
      import Os'File'Metadata { Metadata }
      import Os'Process

      main = \(ctx context) none {
        m = File'Metadata(ctx.Os, "foo")

        c = if m = m as Metadata {
          if m.Size == 3 {
            0
          } else {
            1
          }
        } else {
          1
        }

        Process'Exit(ctx.Os, c)
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
      import Os'Context { Context }
      import Os'File
      import Os'Process
      import Os'Time

      main = \(ctx context) none {
        if m = run(ctx.Os) as none {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }

      run = \(ctx Context) none | error {
        File'Write(ctx, File'StdOut(), Number'String(Time'Now(ctx)))?

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Sleep
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'Time

      main = \(ctx context) none {
        Time'Sleep(ctx.Os, 1)
      }
      """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Exit a process
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'Process

      main = \(ctx context) none {
        Process'Exit(ctx.Os, 42)
      }
      """
    When I successfully run `pen build`
    Then I run `./app`
    And the exit status should be 42
