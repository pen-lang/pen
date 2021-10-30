Feature: List
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///os"
      }
    }
    """

  Scenario: Force multiple elements of a list
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    main = \(ctx Context) number {
      if [x, ...xs] = [none ...[none none]] {
        x()
      } else {
        none
      }

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Force an element in a list of any type
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    main = \(ctx Context) number {
      if [x, ..._] = [any "foo"] {
        x()

        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`
