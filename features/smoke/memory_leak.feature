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

    main = \(os Os'Os) number {
      main(os)
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

  Scenario: Use a global variable
    Given a file named "Main.pen" with:
    """
    import System'Os

    type foo {
      x number,
    }

    foo = \() foo {
      foo{x: 42}
    }

    main = \(os Os'Os) number {
      _ = foo()

      main(os)
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

    foo = \() foo {
      foo{x: 42}
    }

    main = \(os Os'Os) number {
      _ = foo().x

      main(os)
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

    main = \(os Os'Os) number {
      _ = f("")

      main(os)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`

  Scenario: Shadow a variable in a let expression
    Given a file named "Main.pen" with:
    """
    import System'Os

    type foo {
      x number,
    }

    main = \(os Os'Os) number {
      x = foo{x: 42}
      x = x.x

      main(os)
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

    main = \(os Os'Os) number {
      x = foo{x: 42}
      f = \() number { x.x }

      main(os)
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak_in_loop.sh ./app`
