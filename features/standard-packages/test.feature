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
      Assert'True(true)?

      none
    }
    """
    When I run `pen test`
    Then the exit status should be 0

  Scenario: Make a test fail
    Given a file named "foo.test.pen" with:
    """pen
    import Test'Assert

    Foo = \() none | error {
      Assert'Fail()?

      none
    }
    """
    When I run `pen test`
    Then the exit status should be 1

  Scenario: Check if numbers are equal
    Given a file named "foo.test.pen" with:
    """pen
    import Test'Assert

    Foo = \() none | error {
      Assert'EqualNumbers(42, 42)?

      none
    }
    """
    When I run `pen test`
    Then the exit status should be 0

  Scenario: Check if strings are equal
    Given a file named "foo.test.pen" with:
    """pen
    import Test'Assert

    Foo = \() none | error {
      Assert'EqualStrings("foo", "foo")?

      none
    }
    """
    When I run `pen test`
    Then the exit status should be 0
