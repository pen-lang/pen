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
    import Os'Process

    main = \(ctx context) none {
      if "foo" == "foo" {
        none
      } else {
        Process'Exit(ctx.Os, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
