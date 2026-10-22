use crate::position::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    UnterminatedBlockComment,
    UnterminatedLiteral,
    MalformedRawString,
    MalformedNumber,
    UnknownCharacter,
    InvalidIdentifier,
    ReservedPrefix,
    ReservedGuardedString,
    LifetimeStartsWithNumber,
    Frontmatter,
}

impl LexErrorKind {
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            LexErrorKind::UnterminatedBlockComment => "unterminated block comment",
            LexErrorKind::UnterminatedLiteral => "unterminated literal",
            LexErrorKind::MalformedRawString => "malformed raw string literal",
            LexErrorKind::MalformedNumber => "number literal with no digits",
            LexErrorKind::UnknownCharacter => "character that is not a Rust token",
            LexErrorKind::InvalidIdentifier => "invalid identifier",
            LexErrorKind::ReservedPrefix => "reserved literal prefix",
            LexErrorKind::ReservedGuardedString => "reserved guarded string prefix",
            LexErrorKind::LifetimeStartsWithNumber => "lifetime that starts with a number",
            LexErrorKind::Frontmatter => "frontmatter, which this tool does not read",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LexError {
    pub position: Position,
    pub kind: LexErrorKind,
}
