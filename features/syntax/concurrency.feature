Feature: Concurrency
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "Os": "pen:///os"
      }
    }
    """

  Scenario: Use go syntax
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }

    main = \(ctx context) none {
      f = go \() none { none }

      f()
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
