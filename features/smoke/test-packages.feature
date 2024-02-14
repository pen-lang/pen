Feature: Test packages
  Scenario Outline: Build and test test packages
    Given I run the following script:
      """
      cp -r $PEN_ROOT/test .
      """
    When I cd to "test/<package>"
    Then I successfully run `pen format --check`
    And I successfully run `pen build`
    And I successfully run `pen test`

    Examples:
      | package |
      | prelude |
      | reflect |
