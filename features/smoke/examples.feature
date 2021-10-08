Feature: Examples
  Scenario Outline: Build and test examples
    Given I run the following script:
    """
    cp -r $PEN_ROOT/examples .
    """
    When I cd to "examples/<example>"
    Then I successfully run `pen build`
    # TODO Fix pen test on macOS.
    And I run the following script:
    """
    if ! llvm-config --host-target | grep apple; Then
      pen test
    fi
    """

    Examples:
      | example |
      | console |
      | echo    |
      | ls      |
