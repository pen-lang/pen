Feature: Examples
  Scenario Outline: Build and test examples
    Given I run the following script:
    """
    cp -r $PEN_ROOT/examples .
    """
    When I cd to "examples/<example>"
    Then I successfully run `pen build`
    And I successfully run `pen_test_on_linux.sh`

    Examples:
      | example     |
      | cat         |
      | console     |
      | echo        |
      | hello-world |
      | ls          |
      | udp-server  |
      | yes         |
