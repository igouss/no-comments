use std::ffi::OsString;
use std::path::PathBuf;

use no_comments::cli_args::{parse_args, Command, Format};
use no_comments::usage_error::UsageError;

fn parse(args: &[&str]) -> Result<Command, UsageError> {
    parse_args(args.iter().map(OsString::from).collect())
}

fn check(format: Format, roots: &[&str]) -> Command {
    Command::Check {
        format,
        roots: roots.iter().map(PathBuf::from).collect(),
    }
}

#[test]
fn zero_paths_is_a_usage_error() {
    assert_eq!(
        parse(&[]),
        Err(UsageError::new("no paths given: name the trees to check"))
    );
}

#[test]
fn one_path_checks_in_human_format() {
    assert_eq!(parse(&["src"]), Ok(check(Format::Human, &["src"])));
}

#[test]
fn many_paths_keep_their_order() {
    assert_eq!(
        parse(&["b", "a", "c"]),
        Ok(check(Format::Human, &["b", "a", "c"]))
    );
}

#[test]
fn json_shorthand() {
    assert_eq!(parse(&["--json", "src"]), Ok(check(Format::Json, &["src"])));
}

#[test]
fn format_as_a_separate_value() {
    assert_eq!(
        parse(&["src", "--format", "json"]),
        Ok(check(Format::Json, &["src"]))
    );
}

#[test]
fn format_as_an_attached_value() {
    assert_eq!(
        parse(&["--format=json", "src"]),
        Ok(check(Format::Json, &["src"]))
    );
}

#[test]
fn the_last_format_wins() {
    assert_eq!(
        parse(&["--json", "--format=human", "src"]),
        Ok(check(Format::Human, &["src"]))
    );
}

#[test]
fn format_without_a_value_is_a_usage_error() {
    assert_eq!(
        parse(&["src", "--format"]),
        Err(UsageError::new("--format needs a value: human or json"))
    );
}

#[test]
fn an_unknown_format_is_a_usage_error() {
    assert_eq!(
        parse(&["--format=yaml", "src"]),
        Err(UsageError::new(
            "unknown format yaml: expected human or json"
        ))
    );
}

#[test]
fn an_unknown_flag_is_a_usage_error() {
    assert_eq!(
        parse(&["-x", "src"]),
        Err(UsageError::new("unknown flag -x"))
    );
}

#[test]
fn a_lone_dash_is_a_path() {
    assert_eq!(parse(&["-"]), Ok(check(Format::Human, &["-"])));
}

#[test]
fn after_double_dash_flags_are_paths() {
    assert_eq!(
        parse(&["--", "--json", "-h"]),
        Ok(check(Format::Human, &["--json", "-h"]))
    );
}

#[test]
fn help_wins_over_paths() {
    assert_eq!(parse(&["src", "-h"]), Ok(Command::Help));
}

#[test]
fn long_help() {
    assert_eq!(parse(&["--help"]), Ok(Command::Help));
}

#[test]
fn version() {
    assert_eq!(parse(&["-V"]), Ok(Command::Version));
}
