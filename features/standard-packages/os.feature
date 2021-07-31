Feature: OS
  Background:
    Given a file named "pen.json" with:
    """json
    {
      "dependencies": {
        "System": "file://pen-root/lib/os"
      }
    }
    """

  Scenario: Open a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(os Os'Os) number {
      if f = Os'OpenFile(os, "./foo.txt"); Os'File {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`

  Scenario: Write a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    writeFile = \(os Os'Os) none | error {
      f = Os'OpenWriteFile(os, "./foo.txt")?
      Os'WriteFile(os, f, "foo")?

      none
    }

    main = \(os Os'Os) number {
      if _ = writeFile(os); none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "foo.txt" should contain "foo"
