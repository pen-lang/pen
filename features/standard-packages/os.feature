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

  Scenario: Read a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    readFile = \(os Os'Os) none | error {
      f = Os'OpenFile(os, "foo.txt")?
      d = Os'ReadFile(os, f)?
      f = Os'OpenFileWithOptions(
        os,
        "bar.txt",
        Os'OpenFileOptions{
          ...Os'DefaultOpenFileOptions(),
          Create: true,
          Write: true,
        },
      )?
      Os'WriteFile(os, f, d)?

      none
    }

    main = \(os Os'Os) number {
      if _ = readFile(os); none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "bar.txt" should contain "foo"

  Scenario: Write a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    writeFile = \(os Os'Os) none | error {
      f = Os'OpenFileWithOptions(
        os,
        "./foo.txt",
        Os'OpenFileOptions{...Os'DefaultOpenFileOptions(), Write: true},
      )?

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

  Scenario: Copy a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(os Os'Os) number {
      if _ = Os'CopyFile(os, "foo.txt", "bar.txt"); none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with "foo"
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "bar.txt" should contain "foo"

  Scenario: Remove a file
    Given a file named "Main.pen" with:
    """pen
    import System'Os

    main = \(os Os'Os) number {
      if _ = Os'RemoveFile(os, "foo.txt"); none {
        0
      } else {
        1
      }
    }
    """
    And a file named "foo.txt" with ""
    When I successfully run `pen build`
    Then I successfully run `./app`
    And the file "foo.txt" does not exist
