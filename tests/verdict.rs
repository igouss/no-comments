use std::path::PathBuf;

use no_comments::comment_form::CommentForm;
use no_comments::finding::Finding;
use no_comments::position::Position;
use no_comments::report::{FileReport, FileResult, Report, Verdict};

fn file(result: FileResult) -> FileReport {
    FileReport {
        path: PathBuf::from("a.rs"),
        result,
    }
}

fn comments(count: usize) -> FileResult {
    FileResult::Scanned(vec![
        Finding {
            position: Position { line: 1, column: 1 },
            form: CommentForm::Block,
        };
        count
    ])
}

#[test]
fn exit_codes_are_zero_one_two() {
    assert_eq!(
        [Verdict::Clean, Verdict::Findings, Verdict::Error].map(Verdict::exit_code),
        [0, 1, 2]
    );
}

#[test]
fn zero_findings_in_two_files_is_clean() {
    let report: Report = Report {
        files: vec![file(comments(0)), file(comments(0))],
    };
    assert_eq!(
        (report.verdict(), report.finding_count()),
        (Verdict::Clean, 0)
    );
}

#[test]
fn one_finding_is_findings() {
    let report: Report = Report {
        files: vec![file(comments(0)), file(comments(1))],
    };
    assert_eq!(
        (report.verdict(), report.finding_count()),
        (Verdict::Findings, 1)
    );
}

#[test]
fn findings_are_counted_across_files() {
    let report: Report = Report {
        files: vec![file(comments(2)), file(comments(3))],
    };
    assert_eq!(
        (report.verdict(), report.finding_count()),
        (Verdict::Findings, 5)
    );
}

#[test]
fn an_unreadable_file_outranks_findings() {
    let report: Report = Report {
        files: vec![
            file(comments(2)),
            file(FileResult::Unreadable("denied".to_string())),
        ],
    };
    assert_eq!(
        (report.verdict(), report.finding_count()),
        (Verdict::Error, 2)
    );
}
