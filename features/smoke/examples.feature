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
      | example               |
      | algorithms/fizz-buzz  |
      | algorithms/quick-sort |
      | algorithms/knapsack   |
      | cat                   |
      | console               |
      | echo                  |
      | hello-world           |
      | life-game             |
      | ls                    |
      | tcp-client            |
      | tcp-server            |
      | udp-client            |
      | udp-server            |
      | yes                   |

  Scenario Outline: Test memory leak
    Given I run the following script:
    """
    cp -r $PEN_ROOT/examples .
    """
    When I cd to "examples/<example>"
    Then I successfully run `pen build`
    And I successfully run `check_memory_leak_in_loop.sh ./app`

    Examples:
      | example     |
      | life-game   |
      | yes         |
