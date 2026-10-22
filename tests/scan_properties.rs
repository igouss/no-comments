#![allow(
    clippy::needless_pass_by_value,
    reason = "hegel composites take the test case by value"
)]

use hegel::generators::{self, Generator};
use hegel::TestCase;
use no_comments::comment_form::CommentForm;
use no_comments::finding::Finding;
use no_comments::position::Position;
use no_comments::scan::scan;

const COMMENT_LIKE: [&str; 11] = [
    "//",
    "/*",
    "*/",
    "///",
    "//!",
    "/**",
    "/*!",
    "/* /* */ */",
    "\n// x\n",
    "#\"",
    "\"#",
];

const PUNCTUATION: [&str; 26] = [
    ";", ",", ".", "(", ")", "{", "}", "[", "]", "@", "#", "~", "?", ":", "$", "=", "!", "<", ">",
    "-", "&", "|", "+", "*", "/", "^",
];

const SEPARATORS: [&str; 4] = [" ", "\n", "\t", "  \n  "];

const FORMS: [CommentForm; 6] = [
    CommentForm::Line,
    CommentForm::Block,
    CommentForm::OuterLineDoc,
    CommentForm::InnerLineDoc,
    CommentForm::OuterBlockDoc,
    CommentForm::InnerBlockDoc,
];

fn escaped(content: &str) -> String {
    content
        .chars()
        .map(|character: char| match character {
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            '\'' => "\\'".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            other => other.to_string(),
        })
        .collect()
}

#[hegel::composite]
fn comment_like_text(tc: &TestCase) -> String {
    let pieces: Vec<String> = tc.draw(generators::vecs(hegel::one_of!(
        generators::text().max_size(8),
        generators::sampled_from(COMMENT_LIKE.to_vec()).map(str::to_string),
    )));
    pieces.concat()
}

#[hegel::composite]
fn code_token(tc: &TestCase) -> String {
    tc.draw(hegel::one_of!(
        generators::from_regex(r"[a-z_][a-z0-9_]{0,6}|[A-Z][a-zé]{0,4}"),
        generators::integers::<u32>().map(|number: u32| number.to_string()),
        generators::sampled_from(PUNCTUATION.to_vec()).map(str::to_string),
        comment_like_text().map(|content: String| format!("\"{}\"", escaped(&content))),
    ))
}

fn without(text: String, removed: char) -> String {
    text.chars()
        .filter(|character: &char| *character != removed)
        .collect()
}

fn guarded(body: String, forbidden_starts: &[char]) -> String {
    match body.chars().next() {
        Some(first) if forbidden_starts.contains(&first) => format!(" {body}"),
        Some(_) | None => body,
    }
}

#[hegel::composite]
fn block_body(tc: &TestCase) -> String {
    let outer: String = without(tc.draw(generators::text().max_size(12)), '/');
    let nested: Option<String> = tc.draw(generators::optional(
        generators::text()
            .max_size(6)
            .map(|inner: String| format!("/*{}*/", without(inner, '/'))),
    ));
    match nested {
        Some(inner) => format!("{outer} {inner} {outer}"),
        None => outer,
    }
}

#[hegel::composite]
fn comment(tc: &TestCase, form: CommentForm) -> String {
    let line: String = without(tc.draw(generators::text().max_size(12)), '\n');
    match form {
        CommentForm::Line => {
            let four_slashes: bool = tc.draw(generators::booleans());
            if four_slashes {
                format!("////{line}\n")
            } else {
                format!("//{}\n", guarded(line, &['/', '!']))
            }
        }
        CommentForm::OuterLineDoc => format!("///{}\n", guarded(line, &['/'])),
        CommentForm::InnerLineDoc => format!("//!{line}\n"),
        CommentForm::Block => format!("/*{}*/", guarded(tc.draw(block_body()), &['*', '!'])),
        CommentForm::OuterBlockDoc => {
            let body: String = tc.draw(block_body());
            let guarded_body: String = if body.is_empty() {
                " ".to_string()
            } else {
                guarded(body, &['*'])
            };
            format!("/**{guarded_body}*/")
        }
        CommentForm::InnerBlockDoc => format!("/*!{}*/", tc.draw(block_body())),
    }
}

fn joined(tokens: &[String], separators: &[String]) -> String {
    tokens.iter().zip(separators.iter().cycle()).fold(
        String::new(),
        |text: String, (token, separator): (&String, &String)| text + token + separator,
    )
}

fn position_of_end(prefix: &str) -> Position {
    let line_start: usize = prefix.rfind('\n').map_or(0, |newline: usize| newline + 1);
    Position {
        line: prefix.matches('\n').count() + 1,
        column: prefix[line_start..].chars().count() + 1,
    }
}

#[hegel::test(test_cases = 300)]
fn a_comment_inserted_between_two_tokens_is_reported_at_its_position(tc: TestCase) {
    let tokens: Vec<String> = tc.draw(generators::vecs(code_token()).max_size(30));
    let separators: Vec<String> = tc.draw(
        generators::vecs(generators::sampled_from(SEPARATORS.to_vec()).map(str::to_string))
            .min_size(1)
            .max_size(4),
    );
    let split: usize = tc.draw(generators::integers::<usize>().max_value(tokens.len()));
    let form: CommentForm = tc.draw(generators::sampled_from(FORMS.to_vec()));
    let inserted: String = tc.draw(comment(form));
    let before: String = joined(&tokens[..split], &separators);
    let after: String = joined(&tokens[split..], &separators);
    let source: String = format!("{before}{inserted} {after}");
    tc.note(&format!("source: {source:?}"));
    assert_eq!(scan(&format!("{before}{after}")), Ok(Vec::new()));
    assert_eq!(
        scan(&source),
        Ok(vec![Finding {
            position: position_of_end(&before),
            form,
        }])
    );
}

#[hegel::composite]
fn literal(tc: &TestCase) -> String {
    let content: String = tc.draw(comment_like_text());
    let ascii: String = content.chars().filter(char::is_ascii).collect();
    let extra_hashes: usize = tc.draw(generators::integers::<usize>().max_value(2));
    let hashes: String = "#".repeat(raw_hashes_needed(&content) + extra_hashes);
    let ascii_hashes: String = "#".repeat(raw_hashes_needed(&ascii) + extra_hashes);
    let character: char = tc.draw(hegel::one_of!(
        generators::sampled_from(vec!['/', '*', '!', '"', '\'', '\\']),
        generators::characters(),
    ));
    let byte: char = tc.draw(generators::sampled_from(vec![
        '/', '*', '!', '"', '\'', '\\',
    ]));
    let shapes: Vec<String> = vec![
        format!("\"{}\"", escaped(&content)),
        format!("r{hashes}\"{content}\"{hashes}"),
        format!("b\"{}\"", escaped(&ascii)),
        format!("br{ascii_hashes}\"{ascii}\"{ascii_hashes}"),
        format!("'{}'", escaped(&character.to_string())),
        format!("b'{}'", escaped(&byte.to_string())),
    ];
    tc.draw(generators::sampled_from(shapes))
}

fn raw_hashes_needed(content: &str) -> usize {
    content
        .match_indices('"')
        .map(|(offset, _): (usize, &str)| {
            content[offset + 1..]
                .chars()
                .take_while(|character: &char| *character == '#')
                .count()
                + 1
        })
        .max()
        .unwrap_or(0)
}

#[hegel::test(test_cases = 300)]
fn a_file_of_literals_holding_comment_like_text_has_no_finding(tc: TestCase) {
    let literals: Vec<String> = tc.draw(generators::vecs(literal()).min_size(1).max_size(20));
    let separators: Vec<String> = tc.draw(
        generators::vecs(generators::sampled_from(SEPARATORS.to_vec()).map(str::to_string))
            .min_size(1)
            .max_size(4),
    );
    let source: String = joined(&literals, &separators);
    tc.note(&format!("source: {source:?}"));
    assert_eq!(scan(&source), Ok(Vec::new()));
}
