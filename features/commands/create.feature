Feature: Package creator
  Scenario: Create an application package
    Given I successfully run `pen create foo`
    And I cd to "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Create a library package
    Given I successfully run `pen create --library foo`
    And I cd to "foo"
    When I run `pen build`
    Then the exit status should be 0
