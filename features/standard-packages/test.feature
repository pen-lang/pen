Feature: Test
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {
          "Test": "pen:///test"
        }
      }
      """

  Scenario: Check if a condition is true
    Given a file named "foo.test.pen" with:
      """pen
      import Test'Assert

      Foo = \() none | error {
        Assert'True(true)
      }
      """
    When I run `pen test`
    Then the exit status should be 0

  Scenario: Check if a value is an error
    Given a file named "foo.test.pen" with:
      """pen
      import Test'Assert

      Foo = \() none | error {
        Assert'Error(error(none))
      }
      """
    When I run `pen test`
    Then the exit status should be 0

  Scenario: Make a test fail
    Given a file named "foo.test.pen" with:
      """pen
      import Test'Assert

      Foo = \() none | error {
        Assert'Fail()
      }
      """
    When I run `pen test`
    Then the exit status should be 1
