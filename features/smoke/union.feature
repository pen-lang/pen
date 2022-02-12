Feature: Union
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "Os": "pen:///os"
      }
    }
    """

  Scenario: Downcast a union to a list
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }

    main = \(ctx context) none {
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
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
