Feature: List
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///lib/os"
      }
    }
    """

  Scenario: Force multiple elements of a list
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(ctx Os'Context) number {
      if [x, ...xs] = [none; ...[none; none]] {
        x()
      } else {
        none
      }

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
