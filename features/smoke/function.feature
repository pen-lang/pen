Feature: Function
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///os"
      }
    }
    """

  Scenario: Reference an inner closure in nested closures
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    f = \(x number) \() number {
      \() number {
        if x == 0 {
          0
        } else {
          # This should have no effect. But it gets into an infinite loop
          # because it's actually calling the innermost closure!
          f(x - 1)

          0
        }
      }
    }

    main = \(ctx Context) number {
      f(1)()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Shadow a variable of a list in an outer scope
    Given a file named "main.pen" with:
    """pen
    f = \() [none] {
      x = [none]

      [none \(x none) none { x }(none)]
    }

    main = \(ctx Context) number {
      f()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
