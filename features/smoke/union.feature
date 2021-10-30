Feature: Union
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "pen:///os"
      }
    }
    """

  Scenario: Downcast a union to a list
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    main = \(ctx Context) number {
      x = if true {
        [none]
      } else {
        none
      }

      if x = x as [none] {
        none
      } else {
        none
      }

      0
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
