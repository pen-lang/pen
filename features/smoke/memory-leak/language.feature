Feature: Language
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "application",
        "dependencies": {
          "Core": "pen:///core",
          "Os": "pen:///os-sync"
        }
      }
      """

  Scenario: Run hello world
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }
      import Os'File

      main = \(ctx context) none {
        _ = File'Write(ctx.Os, File'StdOut(), "Hello, world!\n")

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Create a record
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        _ = foo{x: 42}

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Deconstruct a record
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        _ = foo{x: 42}.x

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Put a string into a value of any type
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      f = \(x any) any {
        x
      }

      main = \(ctx context) none {
        f("")

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Shadow a variable in a block
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        x = foo{x: 42}
        x = x.x

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Define a function in a let expression with a free variable
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        x = foo{x: 42}
        _ = \() number { x.x }

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Convert a number to a string
    Given a file named "main.pen" with:
      """pen
      import Core'Number
      import Os'Context { Context }

      main = \(ctx context) none {
        Number'String(42)

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Join strings
    Given a file named "main.pen" with:
      """pen
      import Core'String
      import Os'Context { Context }

      main = \(ctx context) none {
        String'Join([string "hello", "world"], " ")

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Drop an unforced list
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        x = foo{x: 42}
        _ = [foo x]

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Drop a forced list
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        x = foo{x: 42}

        if [x, ...xs] = [foo x] {
          x()
        } else {
          none
        }

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Drop an unforced list with no environment
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        [foo foo{x: 42}]

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Drop a forced list with no environment
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      type foo {
        x number
      }

      main = \(ctx context) none {
        if [x, ...xs] = [foo foo{x: 42}] {
          x()
        } else {
          none
        }

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Force an element twice
    Given a file named "main.pen" with:
      """pen
      import Os'Context { Context }

      main = \(ctx context) none {
        xs = [none none]

        if [x, ..._] = xs {
          x()
          x()
        } else {
          none
        }

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Use spawn function
    Given a file named "main.pen" with:
      """pen
      main = \(ctx context) none {
        f = go(\() none { none })

        f()
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Use spawn function with a record
    Given a file named "main.pen" with:
      """pen
      type foo {
        x number
        y number
        z number
      }

      main = \(ctx context) none {
        x = foo{x: 1, y: 2, z: 3}

        f = go(\() none {
          _ = x
          none
        })

        f()
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Use spawn function with a closure
    Given a file named "main.pen" with:
      """pen
      main = \(ctx context) none {
        x = \() none { none }

        f = go(\() none {
          _ = x
          none
        })

        f()
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Use spawn function with a closure with a record
    Given a file named "main.pen" with:
      """pen
      type foo {
        x number
        y number
        z number
      }

      main = \(ctx context) none {
        x = foo{x: 1, y: 2, z: 3}

        y = \() none {
          _ = x
          none
        }

        f = go(\() none {
          _ = y
          none
        })

        f()
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Concatenate strings
    Given a file named "main.pen" with:
      """pen
      main = \(ctx context) none {
        x = "foo"
        y = x + x
        z = y + y

        none
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
