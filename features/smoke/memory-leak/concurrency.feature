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

  Scenario Outline: Use spawn function
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

  Scenario Outline: Use spawn function with a record
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

  Scenario Outline: Use spawn function with a closure
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

  Scenario Outline: Use spawn function with a closure with a record
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

  Scenario Outline: Use race function
    Given a file named "main.pen" with:
      """pen
      import Os'Process

      main = \(ctx context) none {
        xs = race([[none] [none none]])

        if [x, ...xs] = xs {
          <result>
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | result |
      | x()    |
      | none   |

  Scenario Outline: Use race function with a record
    Given a file named "main.pen" with:
      """pen
      import Os'Process

      type foo {
        x number
        y number
      }

      main = \(ctx context) none {
        xs = race([[foo] [foo foo{x: 0, y: 0}]])

        if [x, ...xs] = xs {
          _ = <result>

          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      }
      """
    When I successfully run `pen build`
    Then I successfully run `check_memory_leak.sh ./app`

    Examples:
      | result |
      | x()    |
      | none   |
