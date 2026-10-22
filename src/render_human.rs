use std::fmt::Write;

use crate::finding::Finding;
use crate::report::{FileReport, FileResult, Report, Verdict};
use crate::usage_error::UsageError;

pub struct Rendered {
    pub stdout: String,
    pub stderr: String,
}

#[must_use]
pub fn render_human(outcome: &Result<Report, UsageError>) -> Rendered {
    match outcome {
        Err(error) => Rendered {
            stdout: String::new(),
            stderr: format!(
                "no-comments: error: {}\nRun no-comments --help for usage.\n",
                error.message
            ),
        },
        Ok(report) => render_report(report),
    }
}

fn render_report(report: &Report) -> Rendered {
    let mut stderr: String = String::new();
    for file in &report.files {
        stderr.push_str(&file_lines(file));
    }
    let verdict: Verdict = report.verdict();
    let summary: String = format!(
        "no-comments: {}: {} comment(s) in {} file(s) checked\n",
        verdict.name(),
        report.finding_count(),
        report.files.len()
    );
    match verdict {
        Verdict::Clean => Rendered {
            stdout: summary,
            stderr,
        },
        Verdict::Findings | Verdict::Error => Rendered {
            stdout: String::new(),
            stderr: stderr + &summary,
        },
    }
}

fn file_lines(file: &FileReport) -> String {
    let path: std::path::Display<'_> = file.path.display();
    match &file.result {
        FileResult::Scanned(findings) => {
            findings
                .iter()
                .fold(String::new(), |mut lines: String, finding: &Finding| {
                    let _infallible: std::fmt::Result = writeln!(
                        lines,
                        "{path}:{}:{}: {}",
                        finding.position.line,
                        finding.position.column,
                        finding.form.name()
                    );
                    lines
                })
        }
        FileResult::LexFailed(error) => format!(
            "{path}:{}:{}: error: does not lex: {}\n",
            error.position.line,
            error.position.column,
            error.kind.message()
        ),
        FileResult::Unreadable(message) => format!("{path}: error: cannot read: {message}\n"),
    }
}
