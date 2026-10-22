Feature: the command line and the JSON report

  Scenario: no path argument is a usage error
    Given a git repository
    When no-comments runs with no arguments
    Then the exit status is 2
    And stderr has the line "no-comments: error: no paths given: name the trees to check"

  Scenario: an unknown flag is a usage error
    Given a git repository
    When no-comments runs with "--frobnicate ."
    Then the exit status is 2
    And stderr has the line "no-comments: error: unknown flag --frobnicate"

  Scenario: an unknown format is a usage error
    Given a git repository
    When no-comments runs with "--format yaml ."
    Then the exit status is 2

  Scenario: help is printed and exits 0
    Given a git repository
    When no-comments runs with "--help"
    Then the exit status is 0
    And stdout has the line "Usage: no-comments [--format human|json] [--json] [--] <path>..."

  Scenario: version is printed and exits 0
    Given a git repository
    When no-comments runs with "--version"
    Then the exit status is 0
    And stdout mentions "no-comments 0.1.0"

  Scenario: a path after -- may start with a dash
    Given a git repository
    And the file "-odd.rs" containing:
      """
      // a
      """
    When no-comments runs with "-- -odd.rs"
    Then the exit status is 1
    And stderr has the line "-odd.rs:1:1: line"

  Scenario: the JSON report lists each finding
    Given a git repository
    And the file "src/lib.rs" containing:
      """
      fn f(/* a */) {}
      """
    When no-comments runs with "--json ."
    Then the exit status is 1
    And stdout is a JSON report with verdict "findings" over 1 file(s)
    And the JSON report lists a "block" finding in "src/lib.rs" at 1:6

  Scenario: the JSON report of a clean tree
    Given a git repository
    And the file "src/lib.rs" containing:
      """
      fn f() {}
      """
    When no-comments runs with "--format=json ."
    Then the exit status is 0
    And stdout is a JSON report with verdict "clean" over 1 file(s)

  Scenario: the JSON report names a file that does not lex
    Given a git repository
    And the file "src/lib.rs" containing:
      """
      fn f() { /* open
      """
    When no-comments runs with "--format json ."
    Then the exit status is 2
    And stdout is a JSON report with verdict "error" over 1 file(s)
    And the JSON report lists an error in "src/lib.rs" at 1:10

  Scenario: the JSON report of a usage error
    Given a directory that is not in a git repository
    When no-comments runs with "--json ."
    Then the exit status is 2
    And stdout is a JSON report with verdict "error" over 0 file(s)

  Scenario: two runs print the same bytes
    Given a git repository
    And the file "src/b.rs" containing:
      """
      // b
      """
    And the file "src/a.rs" containing:
      """
      /* a */
      """
    When no-comments runs with "--json ."
    Then a second run prints the same bytes
