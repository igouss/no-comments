use std::path::{Path, PathBuf};

use crate::usage_error::UsageError;

pub trait SourceTree {
    fn rust_files(&self, roots: &[PathBuf]) -> Result<Vec<PathBuf>, UsageError>;
    fn read(&self, path: &Path) -> Result<String, String>;
}
