Feature: Test
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "Test": "pen:///lib/test"
      }
    }
    """

  Scenario: Import a Test package
    Given a file named "FooTest.pen" with:
    """pen
    import Test'Context { Context }

    TestFoo = \(ctx Context) none | error {
      # TODO
      #Assert'True(ctx, true)

      none
    }
    """
    When I run `pen build`
    Then the exit status should be 0
