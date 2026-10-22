Feature: every comment form is reported at its line and column
  The column counts Unicode scalar values from 1, as rustc does, so the
  non-ASCII character before the line comment moves it by one, not by two.

  Background:
    Given a git repository

  Scenario: a line comment
    Given the file "src/lib.rs" containing:
      """
      fn f() {
          let s = "é"; // note
      }
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:2:18: line"
    And stderr has the line "no-comments: findings: 1 comment(s) in 1 file(s) checked"

  Scenario: a line comment with four slashes is a plain line comment
    Given the file "src/lib.rs" containing:
      """
      fn f() {} //// note
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:1:11: line"

  Scenario: a block comment
    Given the file "src/lib.rs" containing:
      """
      fn f(/* note */) {}
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:1:6: block"

  Scenario: a nested block comment is one comment
    Given the file "src/lib.rs" containing:
      """
      fn f() {}
        /* outer /* inner */ still outer */
      fn g() {}
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:2:3: block"
    And stderr has the line "no-comments: findings: 1 comment(s) in 1 file(s) checked"

  Scenario: an outer line doc comment
    Given the file "src/lib.rs" containing:
      """
      struct S {
          /// note
          x: u8,
      }
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:2:5: outer-line-doc"

  Scenario: an inner line doc comment
    Given the file "src/lib.rs" containing:
      """
      mod m {
        //! note
      }
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:2:3: inner-line-doc"

  Scenario: an outer block doc comment
    Given the file "src/lib.rs" containing:
      """
      fn f() {}
         /** note */ fn g() {}
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:2:4: outer-block-doc"

  Scenario: an inner block doc comment
    Given the file "src/lib.rs" containing:
      """
      mod m { /*! note */ }
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:1:9: inner-block-doc"

  Scenario: two comments on one line are two findings
    Given the file "src/lib.rs" containing:
      """
      fn f(/* a */) {} // b
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:1:6: block"
    And stderr has the line "src/lib.rs:1:18: line"
    And stderr has the line "no-comments: findings: 2 comment(s) in 1 file(s) checked"
