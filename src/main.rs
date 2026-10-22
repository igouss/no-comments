use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use no_comments::check::check;
use no_comments::cli_args::{parse_args, Command, Format};
use no_comments::git_tree::GitTree;
use no_comments::help::HELP;
use no_comments::render_human::{render_human, Rendered};
use no_comments::render_json::render_json;
use no_comments::report::{Report, Verdict};
use no_comments::usage_error::UsageError;

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let (rendered, verdict): (Rendered, Verdict) = match parse_args(args) {
        Ok(Command::Help) => (plain(HELP.to_string()), Verdict::Clean),
        Ok(Command::Version) => (
            plain(format!("no-comments {}\n", env!("CARGO_PKG_VERSION"))),
            Verdict::Clean,
        ),
        Ok(Command::Check { format, roots }) => {
            let outcome: Result<Report, UsageError> = check(&roots, &GitTree);
            let verdict: Verdict = match &outcome {
                Ok(report) => report.verdict(),
                Err(_) => Verdict::Error,
            };
            let rendered: Rendered = match format {
                Format::Human => render_human(&outcome),
                Format::Json => plain(render_json(&outcome)),
            };
            (rendered, verdict)
        }
        Err(error) => (render_human(&Err(error)), Verdict::Error),
    };
    let written: std::io::Result<()> = std::io::stdout()
        .write_all(rendered.stdout.as_bytes())
        .and_then(|()| std::io::stderr().write_all(rendered.stderr.as_bytes()));
    match written {
        Ok(()) => ExitCode::from(verdict.exit_code()),
        Err(_) => ExitCode::from(Verdict::Error.exit_code()),
    }
}

fn plain(stdout: String) -> Rendered {
    Rendered {
        stdout,
        stderr: String::new(),
    }
}
