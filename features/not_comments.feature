Feature: text that only looks like a comment is not reported
  Telling these apart takes Rust's lexical rules: a text search for // or /*
  reports every one of them.

  Background:
    Given a git repository

  Scenario: comment-like text in a string literal
    Given the file "src/lib.rs" containing:
      """
      const S: &str = "// not /* a */ comment";
      """
    When no-comments runs with "."
    Then the exit status is 0
    And stdout has the line "no-comments: clean: 0 comment(s) in 1 file(s) checked"

  Scenario: an escaped quote does not end the string early
    Given the file "src/lib.rs" containing:
      """
      const S: &str = "a \" // b";
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: comment-like text in a raw string with no hashes
    Given the file "src/lib.rs" containing:
      """
      const S: &str = r"// /* */";
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: comment-like text in a raw string with one hash
    Given the file "src/lib.rs" containing:
      """
      const S: &str = r#"a "quoted" // b"#;
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: comment-like text in a raw string with three hashes
    Given the file "src/lib.rs" containing:
      """
      const S: &str = r###"a "## // b /* c"###;
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: comment-like text in a byte string and a raw byte string
    Given the file "src/lib.rs" containing:
      """
      const B: &[u8] = b"// /* */";
      const R: &[u8] = br#"// "/*"#;
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: comment-like text in a C string
    Given the file "src/lib.rs" containing:
      """
      const C: &std::ffi::CStr = c"// /* */";
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: slash and star char literals side by side
    Given the file "src/lib.rs" containing:
      """
      const C: [char; 4] = ['/', '/', '/', '*'];
      const D: [u8; 2] = [b'/', b'*'];
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: a shebang line
    Given the file "src/main.rs" containing:
      """
      #!/usr/bin/env run-cargo-script --flag //x
      fn main() {}
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: an inner attribute on the first line is not a shebang
    Given the file "src/lib.rs" containing:
      """
      #![allow(dead_code)]
      fn f() {}
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: an inner attribute on the first line followed by a comment is still checked
    Given the file "src/lib.rs" containing:
      """
      #![allow(dead_code)] // note
      """
    When no-comments runs with "."
    Then the exit status is 1
    And stderr has the line "src/lib.rs:1:22: line"

  Scenario: an outer attribute and a doc attribute
    Given the file "src/lib.rs" containing:
      """
      #[doc = "// documented /* here */"]
      #[derive(Debug)]
      struct S;
      """
    When no-comments runs with "."
    Then the exit status is 0

  Scenario: a lone slash and a division are not comments
    Given the file "src/lib.rs" containing:
      """
      const X: u8 = 8 / 2 * 1;
      """
    When no-comments runs with "."
    Then the exit status is 0
