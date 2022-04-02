Feature: Standard packages
  Scenario Outline: Build and test standard packages
    Given I run the following script:
    """
    cp -r $PEN_ROOT/lib .
    """
    When I cd to "lib/<package>"
    Then I successfully run `pen format --check`
    And I successfully run `pen build`
    And I successfully run `pen_test_on_linux.sh`

    Examples:
      | package |
      | core    |
      | os      |
      | os-sync |
      | test    |
