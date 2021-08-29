Feature: Function
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///lib/os"
      }
    }
    """

  Scenario: Reference an inner closure in nested closures
    Given a file named "Main.pen" with:
    """pen
    import System'Os

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

    main = \(ctx Os'Context) number {
      f(1)()

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
