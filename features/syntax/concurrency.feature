Feature: Concurrency
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

  Scenario: Use spawn function
    Given a file named "main.pen" with:
    """pen
    main = \(ctx context) none {
      f = go(\() none { none })

      f()
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Use race function
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      xs = race([[none] [none none]])

      if [x, ...xs] = xs {
        x()
      } else {
        Process'Exit(ctx.Os, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Use race function with multiple lists
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      xs = race([[none] [none none], [none none]])

      if [x, ...xs] = xs {
        x()
      } else {
        Process'Exit(ctx.Os, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
