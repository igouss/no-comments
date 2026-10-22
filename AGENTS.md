# AGENTS.md — `no-comments`

You are an agent running `no-comments` as a gate, or changing it. This file is your
contract. The README explains the design; this file says what to do.

## 1. The reaction contract

| Exit | Meaning | What you do |
|---|---|---|
| `0` | no comment in any checked file | proceed |
| `1` | comments found | remove each one; move any *why* it carried into the project's docs; re-run |
| `2` | usage error, path outside a git repository or missing, no `.rs` file to check, unreadable file, or a file that does not lex | read stderr (or `errors[]` in JSON), fix the invocation or the file, re-run |

Exit 2 outranks 1: when both occur, every finding from the files that lexed is still
printed. Never read exit 2 as "no comments".

Human format: one line per finding on stderr, `path:line:column: <form>`; the summary line
`no-comments: <verdict>: N comment(s) in M file(s) checked` goes to stdout when clean and to
stderr otherwise. Forms: `line`, `block`, `outer-line-doc`, `inner-line-doc`,
`outer-block-doc`, `inner-block-doc`. Columns are 1-based Unicode scalar values.

JSON (`--json` or `--format json`), one document on stdout, keys sorted, byte-stable:

```json
{"errors":[{"column":10,"line":1,"message":"does not lex: unterminated block comment","path":"src/lib.rs"}],
 "files":1,"findings":[],"verdict":"error"}
```

`findings[]` entries: `path`, `line`, `column`, `form`. `errors[]` entries: `path`,
`line`, `column`, `message`, with `null` where a value does not apply.

## 2. How to run it

```sh
no-comments .                 # a whole repository
no-comments src tests         # several trees; overlaps are checked once
no-comments --json .          # for a machine
```

A path is required; the tool never defaults to the current directory. It checks the
`.rs` files git does not ignore, including untracked ones.

## 3. The rules for changing this repository

These are binding. They come from epic `tools-71h` (tracker `~/code/tools/.beads`) and the
operator's house rules.

1. **No comments, and no exceptions.** The tool has no allow-list, inline suppression or
   baseline, and this repository holds no comment. A case that seems to need an exemption
   is a question for the operator, never a new channel.
2. **The gate is `just check`:** `cargo fmt --check`, `cargo clippy --all-targets -D warnings`
   with `clippy::wildcard_enum_match_arm` at deny, `mutants-triage ledger check`,
   `cargo nextest run`, and `no-comments .` on this repository. Never edit a test, a gate or
   the lint configuration to get a green; show the operator the evidence first.
3. **Red first.** Every new or changed test or check is seen failing on a planted violation
   before it passes. A removed test needs a remaining test proven red on the defect the
   removed one caught.
4. **Test inputs that hold comments are built at test time** in temporary git repositories
   from string literals (see `features/` and `tests/cucumber.rs`). Never commit a fixture
   `.rs` file with a comment: the self-scan would go red, and narrowing the self-scan is not
   an option.
5. **Match exhaustively.** No wildcard arm on an enum; a new `TokenKind` in a lexer upgrade
   must fail to compile in `src/scan.rs`.
6. **Mutation testing** after a change that adds tests: `just mutants-loop no-comments`,
   survivors triaged with `mutants-triage`. A survivor is a test gap until two independent
   refutations admit it to `mutants-equivalence.toml`.
7. **Commits** follow Scoped Commits: `<scope>: <description>`, no `feat:`/`fix:`.

## 4. Known approximations

- Lexing is edition-agnostic: a reserved prefix (`foo#`) is a lex error even in an edition
  2015 file.
- Frontmatter (cargo scripts) is a lex error, not read.
- The lexer's Unicode tables come from `unicode-ident` pinned at 1.0.24 (Unicode 17); see the
  README's Troubleshooting section.
