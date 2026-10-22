pub const HELP: &str = "\
no-comments: fail when a Rust source file holds a comment

Usage: no-comments [--format human|json] [--json] [--] <path>...

Checks every .rs file under the given paths that git does not ignore.
Reports line comments, block comments (nested too) and outer and inner
doc comments, each as path:line:column and its form. Text inside string,
raw string, byte string and char literals, a shebang line and attributes
such as #![...] and #[doc = \"...\"] are not comments.

Options:
  --format human|json   Report format (default: human)
  --json                Same as --format json
  -h, --help            Print this help
  -V, --version         Print the version

Exit status:
  0  no comment found
  1  at least one comment found
  2  usage error, a path outside a git repository, no Rust file to check,
     a file that cannot be read or a file that does not lex;
     findings from the other files are still reported
";
