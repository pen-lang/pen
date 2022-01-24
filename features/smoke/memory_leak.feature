Feature: Memory leak
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "Core": "pen:///core",
        "System": "pen:///os-sync"
      }
    }
    """

  Scenario: Run an infinite loop
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    f = \() none {
      f()
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Run hello world
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }
    import System'File

    main = \(ctx Context) number {
      File'Write(ctx, File'StdOut(), "Hello, world!\n")

      main(ctx)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Create a record
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    f = \() none {
      _ = foo{x: 42}

      f()
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Deconstruct a record
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    f = \() none {
      _ = foo{x: 42}.x

      f()
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Put a string into a value of any type
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    f = \(x any) any {
      x
    }

    g = \() none {
      f("")

      g()
    }

    main = \(ctx Context) number {
      g()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Shadow a variable in a block
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    f = \() none {
      x = foo{x: 42}
      x = x.x

      f()
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Define a function in a let expression with a free variable
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    f = \() none {
      x = foo{x: 42}
      _ = \() number { x.x }

      f()
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Convert a number to a string
    Given a file named "main.pen" with:
    """pen
    import Core'Number
    import System'Context { Context }

    f = \() none {
      Number'String(42)

      f()
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Join strings
    Given a file named "main.pen" with:
    """pen
    import Core'String
    import System'Context { Context }

    f = \() none {
      String'Join([string "hello", "world"], " ")

      f()
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Drop an unforced list
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    main = \(ctx Context) number {
      x = foo{x: 42}

      [foo x]

      main(ctx)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Drop a forced list
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    main = \(ctx Context) number {
      x = foo{x: 42}

      if [x, ...xs] = [foo x] {
        x()
      } else {
        none
      }

      main(ctx)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Drop an unforced list with no environment
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    main = \(ctx Context) number {
      [foo foo{x: 42}]

      main(ctx)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Drop a forced list with no environment
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    type foo {
      x number
    }

    main = \(ctx Context) number {
      if [x, ...xs] = [foo foo{x: 42}] {
        x()
      } else {
        none
      }

      main(ctx)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Force an element twice
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    main = \(ctx Context) number {
      xs = [none none]

      if [x, ..._] = xs {
        x()
        x()
      } else {
        none
      }

      main(ctx)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`
