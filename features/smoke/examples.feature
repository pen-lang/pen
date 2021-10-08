Feature: Examples
  Scenario Outline: Build and test examples
    Given I run the following script:
    """
    cp -r $PEN_ROOT/examples/<example> app
    """
    When I cd to "app"
    Then I successfully run `pen build`
    And I successfully run `pen test`

    Examples:
      | example |
      | console |
      | echo    |
      | ls      |
