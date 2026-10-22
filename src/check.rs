use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::report::{FileReport, FileResult, Report};
use crate::scan::scan;
use crate::source_tree::SourceTree;
use crate::usage_error::UsageError;

pub fn check(roots: &[PathBuf], tree: &dyn SourceTree) -> Result<Report, UsageError> {
    let paths: BTreeSet<PathBuf> = tree.rust_files(roots)?.into_iter().collect();
    if paths.is_empty() {
        return Err(UsageError::new(
            "no Rust files that git does not ignore under the given paths",
        ));
    }
    let files: Vec<FileReport> = paths
        .into_iter()
        .map(|path: PathBuf| {
            let result: FileResult = match tree.read(&path) {
                Ok(source) => match scan(&source) {
                    Ok(findings) => FileResult::Scanned(findings),
                    Err(error) => FileResult::LexFailed(error),
                },
                Err(message) => FileResult::Unreadable(message),
            };
            FileReport { path, result }
        })
        .collect();
    Ok(Report { files })
}
