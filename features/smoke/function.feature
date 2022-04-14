Feature: Function
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "application",
      "dependencies": {
        "Os": "pen:///os"
      }
    }
    """

  Scenario: Reference an inner closure in nested closures
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }

    f = \(x number) \() number {
      \() number {
        if x == 0 {
          0
        } else {
          # This should have no effect. But it gets into an infinite loop
          # when it's actually calling the innermost closure!
          f(x - 1)

          0
        }
      }
    }

    main = \(ctx context) none {
      f(1)()

      none
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

    main = \(ctx context) none {
      f()

      none
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
