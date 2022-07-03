Feature: Concurrency
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "type": "application",
      "dependencies": {
        "Core": "pen:///core",
        "Os": "pen:///os"
      }
    }
    """

  Scenario Outline: Use spawn operation
    Given a file named "main.pen" with:
    """pen
    main = \(ctx context) none {
      f = go(\() none { none })

      <result>
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | result |
      | f()    |
      | none   |

  Scenario Outline: Use spawn operation with a record
    Given a file named "main.pen" with:
    """pen
    type foo {
      x number
      y number
      z number
    }

    main = \(ctx context) none {
      x = foo{x: 1, y: 2, z: 3}

      f = go(\() none {
        _ = x
        none
      })

      <result>
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | result |
      | f()    |
      | none   |

  Scenario Outline: Use spawn operation with a closure
    Given a file named "main.pen" with:
    """pen
    main = \(ctx context) none {
      x = \() none { none }

      f = go(\() none {
        _ = x
        none
      })

      <result>
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | result |
      | f()    |
      | none   |

  Scenario Outline: Use spawn operation with a closure with a record
    Given a file named "main.pen" with:
    """pen
    type foo {
      x number
      y number
      z number
    }

    main = \(ctx context) none {
      x = foo{x: 1, y: 2, z: 3}

      y = \() none {
        _ = x
        none
      }

      f = go(\() none {
        _ = y
        none
      })

      <result>
    }
    """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | result |
      | f()    |
      | none   |
