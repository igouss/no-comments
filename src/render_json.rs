use serde_json::{json, Value};

use crate::finding::Finding;
use crate::report::{FileReport, FileResult, Report, Verdict};
use crate::usage_error::UsageError;

#[must_use]
pub fn render_json(outcome: &Result<Report, UsageError>) -> String {
    let document: Value = match outcome {
        Err(error) => json!({
            "verdict": Verdict::Error.name(),
            "files": 0,
            "findings": [],
            "errors": [{ "path": null, "line": null, "column": null, "message": error.message }],
        }),
        Ok(report) => json!({
            "verdict": report.verdict().name(),
            "files": report.files.len(),
            "findings": report.files.iter().flat_map(findings_of).collect::<Vec<Value>>(),
            "errors": report.files.iter().filter_map(error_of).collect::<Vec<Value>>(),
        }),
    };
    format!("{document}\n")
}

fn findings_of(file: &FileReport) -> Vec<Value> {
    match &file.result {
        FileResult::Scanned(findings) => findings
            .iter()
            .map(|finding: &Finding| {
                json!({
                    "path": file.path.to_string_lossy(),
                    "line": finding.position.line,
                    "column": finding.position.column,
                    "form": finding.form.name(),
                })
            })
            .collect(),
        FileResult::LexFailed(_) | FileResult::Unreadable(_) => Vec::new(),
    }
}

fn error_of(file: &FileReport) -> Option<Value> {
    match &file.result {
        FileResult::Scanned(_) => None,
        FileResult::LexFailed(error) => Some(json!({
            "path": file.path.to_string_lossy(),
            "line": error.position.line,
            "column": error.position.column,
            "message": format!("does not lex: {}", error.kind.message()),
        })),
        FileResult::Unreadable(message) => Some(json!({
            "path": file.path.to_string_lossy(),
            "line": null,
            "column": null,
            "message": format!("cannot read: {message}"),
        })),
    }
}
