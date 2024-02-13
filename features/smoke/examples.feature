Feature: Examples
  Background:
    Given I run the following script:
      """
      cp -r $PEN_ROOT/examples .
      """

  Scenario Outline: Build and test examples
    When I cd to "examples/<example>"
    Then I successfully run `pen format --check`
    And I successfully run `pen build`
    And I successfully run `pen test`

    Examples:
      | example                       |
      | algorithms/fibonacci          |
      | algorithms/fizz-buzz          |
      | algorithms/knapsack           |
      | algorithms/quick-sort         |
      | algorithms/parallel/fibonacci |
      | cat                           |
      | console                       |
      | echo                          |
      | hello-world                   |
      | http-client                   |
      | http-server                   |
      | life-game                     |
      | ls                            |
      | snake                         |
      | sql-client                    |
      | tcp-client                    |
      | tcp-server                    |
      | udp-client                    |
      | udp-server                    |
      | yes                           |

  Scenario: Run HTTP client and server
    Given I cd to "examples/http-server"
    And I successfully run `pen build`
    And I run `./app` in background
    When I cd to "../http-client"
    And I successfully run `pen build`
    Then I successfully run `./app get http://localhost:8080 hello`
    And the stdout from "./app get http://localhost:8080 hello" should contain exactly "hello"

  Scenario: Run TCP client and server
    Given I cd to "examples/tcp-server"
    And I successfully run `pen build`
    And I run `./app localhost:4242` in background
    When I cd to "../tcp-client"
    And I successfully run `pen build`
    Then I successfully run `./app localhost:4242 hello`
    And the stdout from "./app localhost:4242 hello" should contain exactly "hello"

  Scenario: Run UDP client and server
    Given I cd to "examples/udp-server"
    And I successfully run `pen build`
    And I run `./app localhost:4242` in background
    When I cd to "../udp-client"
    And I successfully run `pen build`
    Then I successfully run `./app localhost:4242 hello`
    And the stdout from "./app localhost:4242 hello" should contain exactly "hello"

  Scenario: Run SQL client
    Given I cd to "examples/sql-client"
    And I successfully run `pen build`
    And I successfully run `sqlite3 foo.db 'create table foo (bar int)'`
    When I successfully run `pen build`
    Then I successfully run `./app execute sqlite://foo.db 'insert into foo (bar) values (42)'`
    And I successfully run `./app query sqlite://foo.db 'select * from foo'`
    And the stdout from "./app query sqlite://foo.db 'select * from foo'" should contain exactly "42"
