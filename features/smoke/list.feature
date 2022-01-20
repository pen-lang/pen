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

  Scenario: Compile nested list comprehension
    Given a file named "main.pen" with:
    """pen
    import System'Context { Context }

    f = \(xss [[boolean]]) [[number]] {
      [[number]
        [number if x() { 1 } else { 0 } for x in xs()]
        for xs in xss
      ]
    }

    main = \(ctx Context) number {
      if [x, ..._] = f([[boolean] [boolean true, false]]) {
        x()

        0
      } else {
        1
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

  Scenario: Compile list comprehension with wrong typing
    Given a file named "main.pen" with:
    """pen
    import System'Context

    main = \(ctx Context) number {
      [none y() for y in [none 1]]

      0
    }
    """
    When I run `pen build`
    Then the stderr should contain "types not matched"
