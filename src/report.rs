use std::path::PathBuf;

use crate::finding::Finding;
use crate::lex_error::LexError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileResult {
    Scanned(Vec<Finding>),
    LexFailed(LexError),
    Unreadable(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileReport {
    pub path: PathBuf,
    pub result: FileResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub files: Vec<FileReport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Findings,
    Error,
}

impl Verdict {
    #[must_use]
    pub fn exit_code(self) -> u8 {
        match self {
            Verdict::Clean => 0,
            Verdict::Findings => 1,
            Verdict::Error => 2,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Clean => "clean",
            Verdict::Findings => "findings",
            Verdict::Error => "error",
        }
    }
}

impl Report {
    #[must_use]
    pub fn verdict(&self) -> Verdict {
        let failed: bool = self
            .files
            .iter()
            .any(|file: &FileReport| match &file.result {
                FileResult::Scanned(_) => false,
                FileResult::LexFailed(_) | FileResult::Unreadable(_) => true,
            });
        let commented: bool = self.finding_count() > 0;
        match (failed, commented) {
            (true, _) => Verdict::Error,
            (false, true) => Verdict::Findings,
            (false, false) => Verdict::Clean,
        }
    }

    #[must_use]
    pub fn finding_count(&self) -> usize {
        self.files
            .iter()
            .map(|file: &FileReport| match &file.result {
                FileResult::Scanned(findings) => findings.len(),
                FileResult::LexFailed(_) | FileResult::Unreadable(_) => 0,
            })
            .sum()
    }
}
