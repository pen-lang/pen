Feature: OS
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

  Scenario: Read and write files
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
      """
    And a file named "foo.txt" with "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "bar.txt" should contain "foo"

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

  Scenario: Use go syntax
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      main = \(ctx context) none {
        f = go(\() none { none })

        f()
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
