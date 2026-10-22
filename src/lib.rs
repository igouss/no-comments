#![allow(
    clippy::missing_errors_doc,
    reason = "house rule cm-k7c bans the doc comment this lint asks for"
)]

pub mod check;
pub mod cli_args;
pub mod comment_form;
pub mod finding;
pub mod git_tree;
pub mod help;
pub mod lex_error;
pub mod position;
pub mod render_human;
pub mod render_json;
pub mod report;
pub mod scan;
pub mod source_tree;
pub mod usage_error;
