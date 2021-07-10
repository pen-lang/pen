Feature: Memory leak
  Background:
    Given a file named "pen.json" with:
    """
    {
      "dependencies": {
        "System": "file://pen-root/lib/os"
      }
    }
    """

  Scenario: Run an infinite loop
    Given a file named "Main.pen" with:
    """
    import System'Os

    f = \() none {
      f()
    }

    main = \(os Os'Os) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Run hello world
    Given a file named "Main.pen" with:
    """
    import System'Os

    main = \(os Os'Os) number {
      Os'FdWrite(os, Os'StdOut(), "Hello, world!\n")

      main(os)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Create a record
    Given a file named "Main.pen" with:
    """
    import System'Os

    type foo {
      x number,
    }

    f = \() none {
      _ = foo{x: 42}

      f()
    }

    main = \(os Os'Os) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Deconstruct a record
    Given a file named "Main.pen" with:
    """
    import System'Os

    type foo {
      x number,
    }

    f = \() none {
      _ = foo{x: 42}.x

      f()
    }

    main = \(os Os'Os) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Put a string into a value of any type
    Given a file named "Main.pen" with:
    """
    import System'Os

    f = \(x any) any {
      x
    }

    g = \() none {
      f("")

      g()
    }

    main = \(os Os'Os) number {
      g()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Shadow a variable in a block
    Given a file named "Main.pen" with:
    """
    import System'Os

    type foo {
      x number,
    }

    f = \() none {
      x = foo{x: 42}
      x = x.x

      f()
    }

    main = \(os Os'Os) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Define a function in a let expression with a free variable
    Given a file named "Main.pen" with:
    """
    import System'Os

    type foo {
      x number,
    }

    f = \() none {
      x = foo{x: 42}
      _ = \() number { x.x }

      f()
    }

    main = \(os Os'Os) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`
