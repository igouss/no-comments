use std::ffi::OsString;
use std::path::PathBuf;

use crate::usage_error::UsageError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Human,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Check { format: Format, roots: Vec<PathBuf> },
}

pub fn parse_args(args: Vec<OsString>) -> Result<Command, UsageError> {
    let mut format: Format = Format::Human;
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut flags_ended: bool = false;
    let mut remaining: std::vec::IntoIter<OsString> = args.into_iter();
    while let Some(arg) = remaining.next() {
        let text: Option<&str> = arg.to_str();
        match (flags_ended, text) {
            (false, Some("--")) => flags_ended = true,
            (false, Some("-h" | "--help")) => return Ok(Command::Help),
            (false, Some("-V" | "--version")) => return Ok(Command::Version),
            (false, Some("--json")) => format = Format::Json,
            (false, Some("--format")) => {
                let value: OsString = remaining
                    .next()
                    .ok_or_else(|| UsageError::new("--format needs a value: human or json"))?;
                format = parse_format(&value)?;
            }
            (false, Some(flag)) if flag.starts_with("--format=") => {
                format = parse_format(&OsString::from(&flag["--format=".len()..]))?;
            }
            (false, Some(flag)) if flag.starts_with('-') && flag.len() > 1 => {
                return Err(UsageError::new(format!("unknown flag {flag}")));
            }
            (false | true, _) => roots.push(PathBuf::from(arg)),
        }
    }
    if roots.is_empty() {
        return Err(UsageError::new("no paths given: name the trees to check"));
    }
    Ok(Command::Check { format, roots })
}

fn parse_format(value: &OsString) -> Result<Format, UsageError> {
    match value.to_str() {
        Some("human") => Ok(Format::Human),
        Some("json") => Ok(Format::Json),
        Some(_) | None => Err(UsageError::new(format!(
            "unknown format {}: expected human or json",
            value.to_string_lossy()
        ))),
    }
}
