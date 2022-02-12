Feature: List
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "Os": "pen:///os"
      }
    }
    """

  Scenario: Force multiple elements of a list
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }

    main = \(ctx context) none {
      if [x, ...xs] = [none ...[none none]] {
        x()
      } else {
        none
      }

      none
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Force an element in a list of any type
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }
    import Os'Process

    main = \(ctx context) none {
      if [x, ..._] = [any "foo"] {
        x()

        none
      } else {
        Process'Exit(ctx, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Compile nested list comprehension
    Given a file named "main.pen" with:
    """pen
    import Os'Context { Context }
    import Os'Process

    f = \(xss [[boolean]]) [[number]] {
      [[number]
        [number if x() { 1 } else { 0 } for x in xs()]
        for xs in xss
      ]
    }

    main = \(ctx context) none {
      if [x, ..._] = f([[boolean] [boolean true, false]]) {
        x()

        none
      } else {
        Process'Exit(ctx, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Compile list comprehension with wrong typing
    Given a file named "main.pen" with:
    """pen
    import Os'Context

    main = \(ctx context) none {
      [none y() for y in [none 1]]

      none
    }
    """
    When I run `pen build`
    Then the stderr should contain "types not matched"
