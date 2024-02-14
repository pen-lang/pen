Feature: Standard packages
  Scenario Outline: Build and test standard packages
    Given I run the following script:
      """
      cp -r $PEN_ROOT/packages .
      """
    When I cd to "packages/<package>"
    Then I successfully run `pen format --check`
    And I successfully run `pen build`
    And I successfully run `pen test`

    Examples:
      | package |
      | core    |
      | flag    |
      | html    |
      | http    |
      | json    |
      | os      |
      | os-sync |
      | random  |
      | reflect |
      | regex   |
      | sql     |
      | test    |
