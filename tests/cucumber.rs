#![allow(
    clippy::expect_used,
    reason = "a failed step is a failed scenario; a panic is how cucumber is told"
)]
#![allow(clippy::unused_async, reason = "cucumber requires async steps")]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use serde_json::Value;
use tempfile::TempDir;

#[derive(Debug, Default, World)]
struct NoCommentsWorld {
    sandbox: Option<TempDir>,
    args: Vec<String>,
    git_dir: Option<String>,
    exit: Option<i32>,
    stdout: String,
    stderr: String,
}

impl NoCommentsWorld {
    fn root(&self) -> &Path {
        self.sandbox
            .as_ref()
            .expect("a Given that makes the directory")
            .path()
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root().join(relative)
    }
}

fn git(root: &Path, args: &[&str]) {
    let output: Output = isolated(Command::new("git"), root)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(output.status.success(), "git {args:?}: {output:?}");
}

fn isolated(mut command: Command, root: &Path) -> Command {
    command
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env(
            "GIT_CEILING_DIRECTORIES",
            root.parent().expect("a temp dir has a parent"),
        );
    command
}

fn run(world: &NoCommentsWorld) -> Output {
    let mut command: Command = isolated(
        Command::new(env!("CARGO_BIN_EXE_no-comments")),
        world.root(),
    );
    command.args(&world.args);
    if let Some(git_dir) = &world.git_dir {
        command.env("GIT_DIR", git_dir);
    }
    command.output().expect("spawn no-comments")
}

fn record(world: &mut NoCommentsWorld, output: &Output) {
    world.exit = output.status.code();
    world.stdout = String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8");
    world.stderr = String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8");
}

fn docstring(step: &Step) -> String {
    let text: &str = step.docstring.as_deref().expect("a docstring");
    let body: &str = text.strip_prefix('\n').unwrap_or(text);
    format!("{body}\n")
}

fn json(world: &NoCommentsWorld) -> Value {
    serde_json::from_str(&world.stdout).expect("stdout is one JSON document")
}

#[given("a git repository")]
async fn a_git_repository(world: &mut NoCommentsWorld) {
    let sandbox: TempDir = tempfile::tempdir().expect("make a temp dir");
    git(sandbox.path(), &["init", "-q"]);
    world.sandbox = Some(sandbox);
}

#[given("a directory that is not in a git repository")]
async fn a_plain_directory(world: &mut NoCommentsWorld) {
    world.sandbox = Some(tempfile::tempdir().expect("make a temp dir"));
}

#[given(expr = "the file {string} containing:")]
async fn the_file_containing(world: &mut NoCommentsWorld, relative: String, step: &Step) {
    let path: PathBuf = world.path(&relative);
    std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("make parents");
    std::fs::write(&path, docstring(step)).expect("write the file");
}

#[given(expr = "the file {string} holding bytes that are not UTF-8")]
async fn the_file_not_utf8(world: &mut NoCommentsWorld, relative: String) {
    let path: PathBuf = world.path(&relative);
    std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("make parents");
    std::fs::write(&path, [b'f', b'n', b' ', 0xff, 0xfe, b'\n']).expect("write the file");
}

#[given("the files are staged")]
async fn the_files_are_staged(world: &mut NoCommentsWorld) {
    git(world.root(), &["add", "-A"]);
}

#[given(expr = "the file {string} is deleted")]
async fn the_file_is_deleted(world: &mut NoCommentsWorld, relative: String) {
    std::fs::remove_file(world.path(&relative)).expect("delete the file");
}

#[when(expr = "no-comments runs with {string}")]
async fn no_comments_runs_with(world: &mut NoCommentsWorld, args: String) {
    world.args = args.split_whitespace().map(str::to_string).collect();
    let output: Output = run(world);
    record(world, &output);
}

#[when(expr = "no-comments runs with {string} and GIT_DIR set to {string}")]
async fn no_comments_runs_with_git_dir(world: &mut NoCommentsWorld, args: String, git_dir: String) {
    world.git_dir = Some(git_dir);
    no_comments_runs_with(world, args).await;
}

#[when("no-comments runs with no arguments")]
async fn no_comments_runs_bare(world: &mut NoCommentsWorld) {
    world.args = Vec::new();
    let output: Output = run(world);
    record(world, &output);
}

#[then(expr = "the exit status is {int}")]
async fn the_exit_status_is(world: &mut NoCommentsWorld, expected: i32) {
    assert_eq!(
        world.exit,
        Some(expected),
        "stdout:\n{}\nstderr:\n{}",
        world.stdout,
        world.stderr
    );
}

#[then(expr = "stderr has the line {string}")]
async fn stderr_has_the_line(world: &mut NoCommentsWorld, line: String) {
    assert!(
        world.stderr.lines().any(|seen: &str| seen == line),
        "stderr:\n{}",
        world.stderr
    );
}

#[then(expr = "stdout has the line {string}")]
async fn stdout_has_the_line(world: &mut NoCommentsWorld, line: String) {
    assert!(
        world.stdout.lines().any(|seen: &str| seen == line),
        "stdout:\n{}",
        world.stdout
    );
}

#[then(expr = "stderr mentions {string}")]
async fn stderr_mentions(world: &mut NoCommentsWorld, text: String) {
    assert!(world.stderr.contains(&text), "stderr:\n{}", world.stderr);
}

#[then(expr = "stdout mentions {string}")]
async fn stdout_mentions(world: &mut NoCommentsWorld, text: String) {
    assert!(world.stdout.contains(&text), "stdout:\n{}", world.stdout);
}

#[then(expr = "stderr does not mention {string}")]
async fn stderr_does_not_mention(world: &mut NoCommentsWorld, text: String) {
    assert!(!world.stderr.contains(&text), "stderr:\n{}", world.stderr);
}

#[then(expr = "stdout is a JSON report with verdict {string} over {int} file\\(s)")]
async fn stdout_is_a_json_report(world: &mut NoCommentsWorld, verdict: String, files: u64) {
    let report: Value = json(world);
    assert_eq!(report["verdict"], Value::from(verdict), "{report}");
    assert_eq!(report["files"], Value::from(files), "{report}");
}

#[then(expr = "the JSON report lists a {string} finding in {string} at {int}:{int}")]
async fn the_json_report_lists_a_finding(
    world: &mut NoCommentsWorld,
    form: String,
    path: String,
    line: u64,
    column: u64,
) {
    let report: Value = json(world);
    let expected: Value = serde_json::json!({
        "path": path, "line": line, "column": column, "form": form,
    });
    assert_eq!(report["findings"], Value::Array(vec![expected]), "{report}");
}

#[then(expr = "the JSON report lists an error in {string} at {int}:{int}")]
async fn the_json_report_lists_an_error(
    world: &mut NoCommentsWorld,
    path: String,
    line: u64,
    column: u64,
) {
    let report: Value = json(world);
    let error: &Value = &report["errors"][0];
    assert_eq!(error["path"], Value::from(path), "{report}");
    assert_eq!(error["line"], Value::from(line), "{report}");
    assert_eq!(error["column"], Value::from(column), "{report}");
}

#[then("a second run prints the same bytes")]
async fn a_second_run_prints_the_same_bytes(world: &mut NoCommentsWorld) {
    let second: Output = run(world);
    assert_eq!(
        String::from_utf8(second.stdout).expect("UTF-8"),
        world.stdout
    );
    assert_eq!(
        String::from_utf8(second.stderr).expect("UTF-8"),
        world.stderr
    );
    assert_eq!(second.status.code(), world.exit);
}

#[tokio::test]
async fn acceptance() {
    use cucumber::writer::Stats as _;
    let writer = NoCommentsWorld::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run(concat!(env!("CARGO_MANIFEST_DIR"), "/features"))
        .await;
    assert!(!writer.execution_has_failed(), "a scenario failed");
    assert_eq!(writer.skipped_steps(), 0, "a step was skipped");
}
