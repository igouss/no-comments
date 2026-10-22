# no-comments

> A gate that fails when a Rust source file holds a comment.

```console
$ no-comments .
src/lib.rs:2:18: line
src/lib.rs:7:5: outer-line-doc
no-comments: findings: 2 comment(s) in 14 file(s) checked
$ echo $?
1
```

## The problem it solves

House rule `cm-k7c` bans code comments: names and types carry the *what*, a project's docs
carry the *why*, and only toolchain directives survive. House rule `cm-8gv` says a rule is
enforced by a gate, not by review. Without a gate, the rule holds only as long as every
author and every agent remembers it. `no-comments` is that gate for Rust: put it in a
repository's check recipe and a comment turns the build red.

It sits beside the other `~/code/tools` gates (`hex-lint`, `effect-audit`,
`cargo-regime-check`, `mutants-triage`): one narrow rule, decided mechanically, with exit
codes an agent can branch on.

## Design

1. **Lexical, never textual.** A text search for `//` reports the URL in a string literal.
   `no-comments` runs Rust's own lexer (`ra-ap-rustc_lexer`, the published copy of
   `rustc_lexer`), so a comment is exactly what rustc would call a comment.
2. **Fail closed.** A file the lexer does not fully accept is an error naming the file,
   never a clean result. So is a path outside a git repository, a path that does not exist
   and an empty file set: a run that checked nothing must not read as a run that found
   nothing.
3. **No exceptions.** Rust needs no comment for any toolchain directive, so there is no
   allow-list, no inline suppression and no baseline. A case that seems to need one goes
   to the operator.
4. **It is what it polices.** The repository holds no comment, and its gate runs the tool
   on its own tree. Test inputs that hold comments are built in temporary directories from
   string literals, which the tool reads as literals.

## What it reports

| Form | Spelling | Reported as |
|---|---|---|
| Line comment | `// text`, `//// text` | `line` |
| Block comment, nested or not | `/* text */`, `/* a /* b */ c */` | `block` |
| Outer line doc comment | `/// text` | `outer-line-doc` |
| Inner line doc comment | `//! text` | `inner-line-doc` |
| Outer block doc comment | `/** text */` | `outer-block-doc` |
| Inner block doc comment | `/*! text */` | `inner-block-doc` |

A nested block comment is one finding, at its opening `/*`.

### Not comments

- comment-like text inside a string, raw string (any number of `#`), byte string, raw byte
  string, C string or char literal;
- a shebang line (`#!/usr/bin/env ...` on line 1; `#![...]` on line 1 is an attribute, and
  the line is checked);
- attributes, including `#![...]` and `#[doc = "..."]`.

### Lex errors (exit 2)

A file "does not lex" when the lexer returns any token it did not fully accept: an
unterminated block comment or literal, a malformed raw string, a number with no digits
(`0x`, `1e`), a character that is not a Rust token (`№`), an identifier with an emoji, a
reserved prefix (`foo#`, `'foo#`), a reserved guarded string prefix (`#"`), a lifetime that
starts with a number, or frontmatter (`---` at the top of a cargo script), which this tool
does not read. A file that lexes but does not parse (`fn f( {`) is checked like any other.

### Non-goals

- Comments in other languages, and Rust code outside `.rs` files.
- Files git ignores. The file set is what git would let you commit.
- Deciding whether a comment is useful. Every comment is a finding.

## Usage

```
no-comments [--format human|json] [--json] [--] <path>...
```

| Flag | Meaning |
|---|---|
| `--format human` | Findings and errors on stderr, the summary on stdout when clean and on stderr otherwise (default) |
| `--format json`, `--json` | One JSON document on stdout |
| `--` | Every later argument is a path, even one that starts with `-` |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print the version |

Every path is explicit: with no path argument the tool exits 2 rather than check the
current directory. A path may be a directory or a file; overlapping paths check each file
once.

| Exit | Meaning |
|---|---|
| `0` | No comment in any checked file |
| `1` | At least one comment |
| `2` | Usage error, a path outside a git repository or missing, no `.rs` file to check, a file that cannot be read (not UTF-8, permissions) or a file that does not lex |

When a run has both findings and a file error, the exit status is 2 and every finding from
the files that did lex is still reported.

### Positions

`path:line:column`, both 1-based. The column counts Unicode scalar values, as rustc's
diagnostics do: in `let s = "é"; // x` the comment is at column 14, not 15. A leading byte
order mark is not a column.

### JSON

```json
{"errors":[],"files":1,"findings":[{"column":6,"form":"block","line":1,"path":"src/lib.rs"}],"verdict":"findings"}
```

`verdict` is `clean`, `findings` or `error`. Each `errors` entry has `path`, `line`,
`column` and `message`; `line` and `column` are `null` for a file that cannot be read, and
`path` is also `null` for a usage error. Keys are sorted and the output is byte-identical
from run to run.

## Architecture

```
argv ─▶ cli_args ─▶ check ─▶ SourceTree.rust_files ─▶ git ls-files
                      │
                      ├──▶ SourceTree.read ─▶ scan ─▶ ra-ap-rustc_lexer
                      ▼
                    Report ─▶ render_human | render_json ─▶ stdout, stderr, exit code
```

| Module | Role | Responsibility |
|---|---|---|
| `comment_form.rs` | entity | the six comment forms and their names |
| `position.rs` | entity | line and column from a byte offset |
| `finding.rs` | entity | a comment's form and position |
| `lex_error.rs` | entity | why a file does not lex, and where |
| `usage_error.rs` | entity | an error that is the caller's to fix |
| `scan.rs` | core | source text in, findings or the first lex error out |
| `report.rs` | core | per-file results, the verdict and its exit code |
| `source_tree.rs` | port | the file set and file reading the use case needs |
| `check.rs` | use case | list, read and scan every file into a report |
| `git_tree.rs` | driven adapter | the file set from `git ls-files`, files from disk |
| `render_human.rs`, `render_json.rs` | driven adapters | the two report formats |
| `cli_args.rs`, `help.rs`, `main.rs` | driving shell | argv, help text, I/O and exit status |

`git_tree.rs` runs `git -C <dir> ls-files -z --cached --others --exclude-standard`, removes
what `--deleted` lists (a tracked file deleted from the work tree is not read), and clears
`GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` so the variables a git hook exports do not
redirect the file set.

## The certificate

A green run (exit 0) certifies: every `.rs` file under the given paths that git does not
ignore and that exists in the work tree was read as UTF-8, was accepted by
`ra-ap-rustc_lexer` 0.178.0 without a lex error, and holds no comment token. It certifies
nothing about files git ignores, non-`.rs` files, or Rust that a build script or macro
generates.

## Testing

- `features/*.feature`: the acceptance scenarios, run by `tests/cucumber.rs` against the
  built binary in fresh git repositories under a temporary directory.
- `tests/scan_properties.rs`: `hegeltest` properties. A comment of any form inserted
  between two tokens of a comment-free file is reported at its position; a file built only
  from literals holding comment-like text has no finding.
- `tests/check.rs`: the use case against a fake `SourceTree`.
- `tests/scan.rs`, `tests/position.rs`, `tests/cli_args.rs`, `tests/verdict.rs`: unit
  tests.
- `cargo mutants` through `mutants-triage`: `just mutants-loop no-comments` (it stamps the
  tree by commit, so it needs at least one). The ledger of adjudicated equivalent mutants
  is `mutants-equivalence.toml`. `.cargo/mutants.toml` sets `cap_lints = true`: with
  deny-level rustflags, a mutant that only leaves an unused variable would otherwise fail
  to compile and be reported unviable instead of tested.

## Install and gate

```sh
just install        # cargo install --locked --path .
just check          # fmt, clippy (wildcard_enum_match_arm at deny), ledger, nextest, self-scan
```

## Troubleshooting

- **`unicode-properties and unicode-ident must use the same Unicode version`** when
  building: `ra-ap-rustc_lexer` 0.178.0 asserts at compile time that its two Unicode
  dependencies agree. `unicode-properties` 0.1.4 is on Unicode 17 and `unicode-ident`
  1.0.26 moved to Unicode 18, so `Cargo.toml` pins `unicode-ident = "=1.0.24"`. Drop the
  pin when a lexer release accepts Unicode 18.
- **Exit 2 on a cargo script**: frontmatter is not read; the file is reported as one that
  does not lex.
- **Exit 2 on an edition 2015 file using `foo#`**: reserved prefixes are lexed as errors in
  every edition.

## License

MIT OR Apache-2.0.
