use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use crate::source_tree::SourceTree;
use crate::usage_error::UsageError;

pub struct GitTree;

impl SourceTree for GitTree {
    fn rust_files(&self, roots: &[PathBuf]) -> Result<Vec<PathBuf>, UsageError> {
        let mut files: Vec<PathBuf> = Vec::new();
        for root in roots {
            files.extend(rust_files_under(root)?);
        }
        Ok(files)
    }

    fn read(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|error: std::io::Error| error.to_string())
    }
}

fn rust_files_under(root: &Path) -> Result<BTreeSet<PathBuf>, UsageError> {
    let metadata: std::fs::Metadata = std::fs::metadata(root)
        .map_err(|error: std::io::Error| UsageError::new(format!("{}: {error}", root.display())))?;
    let (directory, pathspec): (PathBuf, PathBuf) = if metadata.is_dir() {
        (root.to_path_buf(), PathBuf::from("."))
    } else {
        let parent: &Path = root.parent().unwrap_or(Path::new(""));
        let name: &std::ffi::OsStr = root
            .file_name()
            .ok_or_else(|| UsageError::new(format!("{}: not a file name", root.display())))?;
        (parent.to_path_buf(), PathBuf::from(name))
    };
    let present: BTreeSet<PathBuf> = git_ls_files(
        &directory,
        &["--cached", "--others", "--exclude-standard"],
        &pathspec,
    )?;
    let deleted: BTreeSet<PathBuf> = git_ls_files(&directory, &["--deleted"], &pathspec)?;
    Ok(present
        .difference(&deleted)
        .filter(|relative: &&PathBuf| {
            relative
                .extension()
                .is_some_and(|extension: &std::ffi::OsStr| extension == "rs")
        })
        .map(|relative: &PathBuf| normalize(&directory.join(relative)))
        .collect())
}

fn git_ls_files(
    directory: &Path,
    selection: &[&str],
    pathspec: &Path,
) -> Result<BTreeSet<PathBuf>, UsageError> {
    let shown_directory: &Path = if directory.as_os_str().is_empty() {
        Path::new(".")
    } else {
        directory
    };
    let output: Output = Command::new("git")
        .arg("-C")
        .arg(shown_directory)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .arg("ls-files")
        .arg("-z")
        .args(selection)
        .arg("--")
        .arg(pathspec)
        .output()
        .map_err(|error: std::io::Error| UsageError::new(format!("cannot run git: {error}")))?;
    if !output.status.success() {
        return Err(UsageError::new(format!(
            "{} is not inside a git repository: {}",
            shown_directory.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let listing: String = String::from_utf8(output.stdout).map_err(|_| {
        UsageError::new(format!(
            "{}: git listed a path that is not UTF-8",
            shown_directory.display()
        ))
    })?;
    Ok(listing
        .split('\0')
        .filter(|entry: &&str| !entry.is_empty())
        .map(PathBuf::from)
        .collect())
}

fn normalize(path: &Path) -> PathBuf {
    path.components()
        .filter(|component: &Component<'_>| *component != Component::CurDir)
        .collect()
}
