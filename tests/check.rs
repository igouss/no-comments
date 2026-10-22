use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use no_comments::check::check;
use no_comments::comment_form::CommentForm;
use no_comments::finding::Finding;
use no_comments::lex_error::{LexError, LexErrorKind};
use no_comments::position::Position;
use no_comments::report::{FileReport, FileResult, Report, Verdict};
use no_comments::source_tree::SourceTree;
use no_comments::usage_error::UsageError;

struct FakeTree {
    listing: Result<Vec<PathBuf>, UsageError>,
    contents: BTreeMap<PathBuf, String>,
}

impl FakeTree {
    fn with_files(files: &[(&str, &str)]) -> FakeTree {
        FakeTree {
            listing: Ok(files
                .iter()
                .map(|(path, _): &(&str, &str)| PathBuf::from(path))
                .collect()),
            contents: files
                .iter()
                .map(|(path, text): &(&str, &str)| (PathBuf::from(path), (*text).to_string()))
                .collect(),
        }
    }
}

impl SourceTree for FakeTree {
    fn rust_files(&self, _roots: &[PathBuf]) -> Result<Vec<PathBuf>, UsageError> {
        self.listing.clone()
    }

    fn read(&self, path: &Path) -> Result<String, String> {
        self.contents
            .get(path)
            .cloned()
            .ok_or_else(|| "gone".to_string())
    }
}

fn roots() -> Vec<PathBuf> {
    vec![PathBuf::from(".")]
}

fn scanned(path: &str, findings: Vec<Finding>) -> FileReport {
    FileReport {
        path: PathBuf::from(path),
        result: FileResult::Scanned(findings),
    }
}

fn line_comment_at(line: usize, column: usize) -> Finding {
    Finding {
        position: Position { line, column },
        form: CommentForm::Line,
    }
}

#[test]
fn zero_files_is_a_usage_error() {
    let outcome: Result<Report, UsageError> = check(&roots(), &FakeTree::with_files(&[]));
    assert_eq!(
        outcome,
        Err(UsageError::new(
            "no Rust files that git does not ignore under the given paths"
        ))
    );
}

#[test]
fn a_listing_error_is_passed_through() {
    let tree: FakeTree = FakeTree {
        listing: Err(UsageError::new("not a repository")),
        contents: BTreeMap::new(),
    };
    assert_eq!(
        check(&roots(), &tree),
        Err(UsageError::new("not a repository"))
    );
}

#[test]
fn one_clean_file() {
    let outcome: Result<Report, UsageError> =
        check(&roots(), &FakeTree::with_files(&[("a.rs", "fn a() {}\n")]));
    assert_eq!(
        outcome,
        Ok(Report {
            files: vec![scanned("a.rs", Vec::new())]
        })
    );
    assert_eq!(
        outcome.map(|report: Report| report.verdict()),
        Ok(Verdict::Clean)
    );
}

#[test]
fn many_files_are_sorted_and_listed_once() {
    let tree: FakeTree = FakeTree {
        listing: Ok(vec![
            PathBuf::from("b.rs"),
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
        ]),
        contents: FakeTree::with_files(&[("a.rs", "// a\n"), ("b.rs", "fn b() {}\n")]).contents,
    };
    let outcome: Result<Report, UsageError> = check(&roots(), &tree);
    assert_eq!(
        outcome,
        Ok(Report {
            files: vec![
                scanned("a.rs", vec![line_comment_at(1, 1)]),
                scanned("b.rs", Vec::new())
            ]
        })
    );
    assert_eq!(
        outcome.map(|report: Report| report.verdict()),
        Ok(Verdict::Findings)
    );
}

#[test]
fn an_unreadable_file_is_an_error_beside_the_findings_of_the_others() {
    let tree: FakeTree = FakeTree {
        listing: Ok(vec![PathBuf::from("a.rs"), PathBuf::from("gone.rs")]),
        contents: FakeTree::with_files(&[("a.rs", "// a\n")]).contents,
    };
    let outcome: Result<Report, UsageError> = check(&roots(), &tree);
    assert_eq!(
        outcome,
        Ok(Report {
            files: vec![
                scanned("a.rs", vec![line_comment_at(1, 1)]),
                FileReport {
                    path: PathBuf::from("gone.rs"),
                    result: FileResult::Unreadable("gone".to_string()),
                },
            ]
        })
    );
    assert_eq!(
        outcome.map(|report: Report| (report.verdict(), report.finding_count())),
        Ok((Verdict::Error, 1))
    );
}

#[test]
fn a_file_that_does_not_lex_is_an_error() {
    let outcome: Result<Report, UsageError> =
        check(&roots(), &FakeTree::with_files(&[("a.rs", "/* open\n")]));
    assert_eq!(
        outcome,
        Ok(Report {
            files: vec![FileReport {
                path: PathBuf::from("a.rs"),
                result: FileResult::LexFailed(LexError {
                    position: Position { line: 1, column: 1 },
                    kind: LexErrorKind::UnterminatedBlockComment,
                }),
            }]
        })
    );
    assert_eq!(
        outcome.map(|report: Report| report.verdict()),
        Ok(Verdict::Error)
    );
}
