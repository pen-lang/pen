Feature: Examples
  Scenario Outline: Build and test examples
    Given I run the following script:
    """
    cp -r $PEN_ROOT/examples .
    """
    When I cd to "examples/<example>"
    Then I successfully run `pen build`
    And I successfully run `pen test`

    Examples:
      | example |
      | console |
      | echo    |
      | ls      |
