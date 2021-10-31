Feature: Testing
  Background:
    Given a file named "pen.json" with:
    """json
    {
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

  # TODO Move this to commands/test.feature when pen test is fixed on macOS.
  Scenario: Run multiple tests
    Given a file named "Foo.test.pen" with:
    """pen
    import Test'Assert
    import 'Foo

    Add = \() none | error {
      Assert'True(Foo'Add(41, 1) == 42)
    }

    AddMore = \() none | error {
      Assert'True(Foo'Add(40, 2) == 42)
    }
    """
    When I run `pen_test_on_linux.sh`
    Then the exit status should be 0
