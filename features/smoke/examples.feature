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

  Scenario: Run HTTP client and server
    Given I run the following script:
    """
    cp -r $PEN_ROOT/examples .
    """
    And I cd to "examples/http-server"
    And I successfully run `pen build`
    And I run `./app` in background
    When I cd to "../http-client"
    And I successfully run `pen build`
    Then I successfully run `./app get http://localhost:8080 hello`
    And the stdout from "./app get http://localhost:8080 hello" should contain exactly "hello"
