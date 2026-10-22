Feature: the files checked are the .rs files under the given paths that git does not ignore

  Scenario: zero Rust files is a usage error, not a clean result
    Given a git repository
    And the file "README.md" containing:
      """
      // not Rust
      """
    When no-comments runs with "."
    Then the exit status is 2
    And stderr has the line "no-comments: error: no Rust files that git does not ignore under the given paths"

  Scenario: one clean file
    Given a git repository
    And the file "src/lib.rs" containing:
      """
      fn f() {}
      """
    When no-comments runs with "."
    Then the exit status is 0
    And stdout has the line "no-comments: clean: 0 comment(s) in 1 file(s) checked"

  Scenario: many files, findings in two of three
    Given a git repository
    And the file "src/a.rs" containing:
      """
      // a
      """
    And the file "src/b.rs" containing:
      """
      fn b() {}
      """
    And the file "src/deep/c.rs" containing:
      """
      fn c() {} /* c */
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/a.rs:1:1: line"
    And stderr has the line "src/deep/c.rs:1:11: block"
    And stderr has the line "no-comments: findings: 2 comment(s) in 3 file(s) checked"

  Scenario: an ignored file is not checked
    Given a git repository
    And the file ".gitignore" containing:
      """
      /generated.rs
      """
    And the file "generated.rs" containing:
      """
      // generated
      """
    And the file "src/lib.rs" containing:
      """
      fn f() {}
      """
    When no-comments runs with "."
    Then the exit status is 0
    And stdout has the line "no-comments: clean: 0 comment(s) in 1 file(s) checked"

  Scenario: an ignored file named on its own leaves nothing to check
    Given a git repository
    And the file ".gitignore" containing:
      """
      /generated.rs
      """
    And the file "generated.rs" containing:
      """
      // generated
      """
    When no-comments runs with "generated.rs"
    Then the exit status is 2

  Scenario: a tracked file is checked, and a tracked file deleted from the work tree is not read
    Given a git repository
    And the file "src/kept.rs" containing:
      """
      // kept
      """
    And the file "src/gone.rs" containing:
      """
      fn gone() {}
      """
    And the files are staged
    And the file "src/gone.rs" is deleted
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/kept.rs:1:1: line"
    And stderr has the line "no-comments: findings: 1 comment(s) in 1 file(s) checked"

  Scenario: a file named directly, and a subdirectory, are checked alone
    Given a git repository
    And the file "src/a.rs" containing:
      """
      // a
      """
    And the file "other/b.rs" containing:
      """
      // b
      """
    When no-comments runs with "src/a.rs"
    Then the exit status is 1
    And stderr has the line "src/a.rs:1:1: line"
    And stderr does not mention "other/b.rs"

  Scenario: overlapping paths check each file once
    Given a git repository
    And the file "src/a.rs" containing:
      """
      // a
      """
    When no-comments runs with ". src src/a.rs"
    Then the exit status is 1
    And stderr has the line "no-comments: findings: 1 comment(s) in 1 file(s) checked"

  Scenario: a path outside a git repository is a usage error
    Given a directory that is not in a git repository
    And the file "src/lib.rs" containing:
      """
      // a
      """
    When no-comments runs with "."
    Then the exit status is 2
    And stderr mentions "is not inside a git repository"

  Scenario: a path that does not exist is a usage error
    Given a git repository
    And the file "src/lib.rs" containing:
      """
      fn f() {}
      """
    When no-comments runs with ". missing"
    Then the exit status is 2
    And stderr mentions "missing:"

  Scenario: git variables inherited from a hook do not redirect the file set
    Given a git repository
    And the file "src/lib.rs" containing:
      """
      // a
      """
    When no-comments runs with "src" and GIT_DIR set to "elsewhere/.git"
    Then the exit status is 1
    And stderr has the line "src/lib.rs:1:1: line"
