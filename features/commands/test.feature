Feature: Testing packages
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
    And a file named "Foo.pen" with:
      """pen
      Add = \(x number, y number) number {
        x + y
      }
      """

  Scenario: Test a module
    Given a file named "Foo.test.pen" with:
      """pen
      import Test'Assert
      import 'Foo

      Add = \() none | error {
        Assert'Equal(Foo'Add(41, 1), 42)
      }
      """
    When I run `pen test`
    Then the exit status should be 0
    And the stdout should contain "OK"

  Scenario: Fail to test a module
    Given a file named "Foo.test.pen" with:
      """pen
      import Test'Assert
      import 'Foo

      Add = \() none | error {
        Assert'Equal(Foo'Add(41, 0), 42)
      }
      """
    When I run `pen test`
    Then the exit status should not be 0
    And the stdout should contain "FAIL"

  Scenario: Run multiple tests
    Given a file named "Foo.test.pen" with:
      """pen
      import Test'Assert
      import 'Foo

      Add = \() none | error {
        Assert'Equal(Foo'Add(41, 1), 42)
      }

      AddMore = \() none | error {
        Assert'Equal(Foo'Add(40, 2), 42)
      }
      """
    When I successfully run `pen test`
    Then the exit status should be 0

  Scenario: Run no test
    When I run `pen test`
    Then the exit status should be 0

  Scenario: Use a debug function in a test
    Given a file named "Foo.test.pen" with:
      """pen
      Foo = \() none | error {
        debug("hello")
      }
      """
    When I run `pen test`
    Then the exit status should be 0
