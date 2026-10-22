use no_comments::comment_form::CommentForm;
use no_comments::finding::Finding;
use no_comments::lex_error::{LexError, LexErrorKind};
use no_comments::position::Position;
use no_comments::scan::scan;

fn malformed(line: usize, column: usize, kind: LexErrorKind) -> Result<Vec<Finding>, LexError> {
    Err(LexError {
        position: Position { line, column },
        kind,
    })
}

fn found(line: usize, column: usize, form: CommentForm) -> Vec<Finding> {
    vec![Finding {
        position: Position { line, column },
        form,
    }]
}

#[test]
fn an_empty_file_has_no_finding() {
    assert_eq!(scan(""), Ok(Vec::new()));
}

#[test]
fn a_byte_order_mark_is_not_a_column() {
    assert_eq!(scan("\u{feff}// a\n"), Ok(found(1, 1, CommentForm::Line)));
}

#[test]
fn a_shebang_line_is_skipped_and_the_next_line_is_checked() {
    assert_eq!(
        scan("#!/bin/sh // x\n  // y\n"),
        Ok(found(2, 3, CommentForm::Line))
    );
}

#[test]
fn columns_count_characters_not_bytes() {
    assert_eq!(
        scan("const S: &str = \"日本\"; /* x */"),
        Ok(found(1, 23, CommentForm::Block))
    );
}

#[test]
fn crlf_line_endings() {
    assert_eq!(
        scan("fn f() {}\r\n  // x\r\n"),
        Ok(found(2, 3, CommentForm::Line))
    );
}

#[test]
fn an_unterminated_nested_block_comment_does_not_lex() {
    assert_eq!(
        scan("fn f() {}\n/* a /* b */\n"),
        malformed(2, 1, LexErrorKind::UnterminatedBlockComment)
    );
}

#[test]
fn an_unterminated_char_does_not_lex() {
    assert_eq!(
        scan("const C: char = '/;"),
        malformed(1, 17, LexErrorKind::UnterminatedLiteral)
    );
}

#[test]
fn an_unterminated_byte_does_not_lex() {
    assert_eq!(
        scan("b'a"),
        malformed(1, 1, LexErrorKind::UnterminatedLiteral)
    );
}

#[test]
fn an_unterminated_byte_string_does_not_lex() {
    assert_eq!(
        scan("b\"a"),
        malformed(1, 1, LexErrorKind::UnterminatedLiteral)
    );
}

#[test]
fn an_unterminated_c_string_does_not_lex() {
    assert_eq!(
        scan("c\"a"),
        malformed(1, 1, LexErrorKind::UnterminatedLiteral)
    );
}

#[test]
fn an_unterminated_raw_string_does_not_lex() {
    assert_eq!(
        scan("r#\"a\""),
        malformed(1, 1, LexErrorKind::MalformedRawString)
    );
}

#[test]
fn an_unterminated_raw_byte_string_does_not_lex() {
    assert_eq!(
        scan("br#\"a\""),
        malformed(1, 1, LexErrorKind::MalformedRawString)
    );
}

#[test]
fn an_unterminated_raw_c_string_does_not_lex() {
    assert_eq!(
        scan("cr#\"a\""),
        malformed(1, 1, LexErrorKind::MalformedRawString)
    );
}

#[test]
fn a_raw_c_string_lexes() {
    assert_eq!(scan("cr#\"// a\"#"), Ok(Vec::new()));
}

#[test]
fn a_hex_prefix_without_digits_does_not_lex() {
    assert_eq!(scan("0x"), malformed(1, 1, LexErrorKind::MalformedNumber));
}

#[test]
fn an_exponent_without_digits_does_not_lex() {
    assert_eq!(scan("1e"), malformed(1, 1, LexErrorKind::MalformedNumber));
}

#[test]
fn numbers_with_digits_lex() {
    assert_eq!(scan("0x1F 1.5e3 7"), Ok(Vec::new()));
}

#[test]
fn an_emoji_identifier_does_not_lex() {
    assert_eq!(
        scan("let a🦀 = 1;"),
        malformed(1, 5, LexErrorKind::InvalidIdentifier)
    );
}

#[test]
fn a_reserved_prefix_does_not_lex() {
    assert_eq!(
        scan("foo#bar"),
        malformed(1, 1, LexErrorKind::ReservedPrefix)
    );
}

#[test]
fn a_reserved_lifetime_prefix_does_not_lex() {
    assert_eq!(
        scan("'foo#bar"),
        malformed(1, 1, LexErrorKind::ReservedPrefix)
    );
}

#[test]
fn a_guarded_string_prefix_does_not_lex() {
    assert_eq!(
        scan("#\"a\"#"),
        malformed(1, 1, LexErrorKind::ReservedGuardedString)
    );
}

#[test]
fn a_lifetime_starting_with_a_number_does_not_lex() {
    assert_eq!(
        scan("'1a"),
        malformed(1, 1, LexErrorKind::LifetimeStartsWithNumber)
    );
}

#[test]
fn lifetimes_and_raw_identifiers_lex() {
    assert_eq!(
        scan("fn f<'a>(r#type: &'a u8, x: &'r#loop u8) {}"),
        Ok(Vec::new())
    );
}

#[test]
fn frontmatter_is_not_read() {
    assert_eq!(
        scan("---\n[package]\n---\nfn main() {}\n"),
        malformed(1, 1, LexErrorKind::Frontmatter)
    );
}

#[test]
fn the_first_lex_error_is_reported() {
    assert_eq!(
        scan("// a\n№ \"open"),
        malformed(2, 1, LexErrorKind::UnknownCharacter)
    );
}
