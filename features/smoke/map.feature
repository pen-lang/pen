Feature: Map
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

  Scenario: Get an entry
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      xs = {string: number "foo": 42}

      if x = xs["foo"] {
        if x == 42 {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      } else {
        Process'Exit(ctx.Os, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Get an entry with a union key
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      xs = {string | none: number "foo": 42}

      if x = xs["foo"] {
        if x == 42 {
          none
        } else {
          Process'Exit(ctx.Os, 1)
        }
      } else {
        Process'Exit(ctx.Os, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Fail to get an entry
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      xs = {string: number}

      if _ = xs["foo"] {
        Process'Exit(ctx.Os, 1)
      } else {
        none
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Create a map with unions
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      _ = {string | none: number | none "foo": 42, none: none}

      none
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Get an entry in a merged map
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      xs = {string: number
        ...{string: number "foo": 40},
        ...{string: number "bar": 2},
      }
      fail = \() none { Process'Exit(ctx.Os, 1) }

      if x = xs["foo"] {
        if y = xs["bar"] {
          if x + y == 42 {
            none
          } else {
            fail()
          }
        } else {
          fail()
        }
      } else {
        fail()
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Compare maps
    Given a file named "main.pen" with:
    """pen
    import Os'Process

    main = \(ctx context) none {
      if {string: none "foo": none} == {string: none "foo": none} {
        none
      } else {
        Process'Exit(ctx.Os, 1)
      }
    }
    """
    When I successfully run `pen build`
    Then I successfully run `./app`
