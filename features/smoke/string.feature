Feature: String
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "Os": "pen:///os"
      }
    }
    """

  Scenario: Compare strings
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }

    main = \(ctx Context) number {
      if "foo" == "foo" {
        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
