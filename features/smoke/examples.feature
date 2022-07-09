Feature: Examples
  Scenario Outline: Build and test examples
    Given I run the following script:
    """
    cp -r $PEN_ROOT/examples .
    """
    When I cd to "examples/<example>"
    Then I successfully run `pen format --check`
    And I successfully run `pen build`
    And I successfully run `pen test`

    Examples:
      | example               |
      | algorithms/fizz-buzz  |
      | algorithms/knapsack   |
      | algorithms/quick-sort |
      | cat                   |
      | console               |
      | echo                  |
      | hello-world           |
      | http-client           |
      | http-server           |
      | life-game             |
      | ls                    |
      | sql-client            |
      | tcp-client            |
      | tcp-server            |
      | udp-client            |
      | udp-server            |
      | yes                   |
