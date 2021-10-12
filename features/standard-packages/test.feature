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

  Scenario: Use a True function
    Given a file named "FooTest.pen" with:
    """pen
    import Test'Assert

    TestFoo = \() none | error {
      Assert'True(true)?

      none
    }
    """
    When I run `pen build`
    Then the exit status should be 0
