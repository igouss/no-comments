Feature: a file the tool cannot fully read or lex is an error naming the file

  Background:
    Given a git repository

  Scenario: an unterminated block comment does not lex
    Given the file "src/lib.rs" containing:
      """
      fn f() {}
      fn g() { /* open
      """
    When no-comments runs with "."
    Then the exit status is 2
    And stderr has the line "src/lib.rs:2:10: error: does not lex: unterminated block comment"

  Scenario: an unterminated string does not lex
    Given the file "src/lib.rs" containing:
      """
      const S: &str = "open;
      """
    When no-comments runs with "."
    Then the exit status is 2
    And stderr has the line "src/lib.rs:1:17: error: does not lex: unterminated literal"

  Scenario: a character that is not a Rust token does not lex
    Given the file "src/lib.rs" containing:
      """
      fn f() { № }
      """
    When no-comments runs with "."
    Then the exit status is 2
    And stderr has the line "src/lib.rs:1:10: error: does not lex: character that is not a Rust token"

  Scenario: a file that lexes but does not parse is checked like any other
    Given the file "src/lib.rs" containing:
      """
      fn f( {
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: a file that is not UTF-8 is an error naming the file
    Given the file "src/lib.rs" holding bytes that are not UTF-8
    When no-comments runs with "."
    Then the exit status is 2
    And stderr mentions "src/lib.rs: error: cannot read:"

  Scenario: findings in the other files are still reported beside an error
    Given the file "src/bad.rs" containing:
      """
      fn f() { /* open
      """
    And the file "src/good.rs" containing:
      """
      // note
      """
    When no-comments runs with "."
    Then the exit status is 2
    And stderr has the line "src/bad.rs:1:10: error: does not lex: unterminated block comment"
    And stderr has the line "src/good.rs:1:1: line"
    And stderr has the line "no-comments: error: 1 comment(s) in 2 file(s) checked"
