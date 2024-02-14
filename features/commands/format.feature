Feature: Formatting module files
  Background:
    Given a file named "pen.json" with:
      """json
      {
        "type": "library",
        "dependencies": {}
      }
      """

  Scenario: Format module files
    Given a file named "Foo.pen" with:
      """pen
      Foo = \() none {

        none
      }
      """
    When I successfully run `pen format`
    Then a file named "Foo.pen" should contain exactly:
      """pen
      Foo = \() none {
        none
      }
      """

  Scenario: Check if module files are formatted
    Given a file named "Foo.pen" with:
      """pen
      Foo = \() none {

        none
      }
      """
    When I run `pen format --checked`
    Then the exit status should not be 0
