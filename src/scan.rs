use ra_ap_rustc_lexer::{
    strip_shebang, Cursor, DocStyle, FrontmatterAllowed, LiteralKind, TokenKind,
};

use crate::comment_form::CommentForm;
use crate::finding::Finding;
use crate::lex_error::{LexError, LexErrorKind};
use crate::position::LineIndex;

enum Lexeme {
    Comment(CommentForm),
    Malformed(LexErrorKind),
    Code,
    End,
}

pub fn scan(source: &str) -> Result<Vec<Finding>, LexError> {
    let text: &str = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: LineIndex<'_> = LineIndex::new(text);
    let shebang: usize = strip_shebang(text).unwrap_or(0);
    let mut cursor: Cursor<'_> = Cursor::new(&text[shebang..], FrontmatterAllowed::Yes);
    let mut findings: Vec<Finding> = Vec::new();
    loop {
        let start: usize = text.len() - cursor.as_str().len();
        let kind: TokenKind = cursor.advance_token().kind;
        match classify(kind) {
            Lexeme::Comment(form) => findings.push(Finding {
                position: lines.position(start),
                form,
            }),
            Lexeme::Malformed(kind) => {
                return Err(LexError {
                    position: lines.position(start),
                    kind,
                })
            }
            Lexeme::Code => {}
            Lexeme::End => return Ok(findings),
        }
    }
}

fn classify(kind: TokenKind) -> Lexeme {
    match kind {
        TokenKind::LineComment { doc_style } => Lexeme::Comment(match doc_style {
            None => CommentForm::Line,
            Some(DocStyle::Outer) => CommentForm::OuterLineDoc,
            Some(DocStyle::Inner) => CommentForm::InnerLineDoc,
        }),
        TokenKind::BlockComment {
            terminated: false, ..
        } => Lexeme::Malformed(LexErrorKind::UnterminatedBlockComment),
        TokenKind::BlockComment {
            doc_style,
            terminated: true,
        } => Lexeme::Comment(match doc_style {
            None => CommentForm::Block,
            Some(DocStyle::Outer) => CommentForm::OuterBlockDoc,
            Some(DocStyle::Inner) => CommentForm::InnerBlockDoc,
        }),
        TokenKind::Literal { kind, .. } => classify_literal(kind),
        TokenKind::Frontmatter { .. } => Lexeme::Malformed(LexErrorKind::Frontmatter),
        TokenKind::InvalidIdent => Lexeme::Malformed(LexErrorKind::InvalidIdentifier),
        TokenKind::UnknownPrefix | TokenKind::UnknownPrefixLifetime => {
            Lexeme::Malformed(LexErrorKind::ReservedPrefix)
        }
        TokenKind::GuardedStrPrefix => Lexeme::Malformed(LexErrorKind::ReservedGuardedString),
        TokenKind::Lifetime {
            starts_with_number: true,
        } => Lexeme::Malformed(LexErrorKind::LifetimeStartsWithNumber),
        TokenKind::Unknown => Lexeme::Malformed(LexErrorKind::UnknownCharacter),
        TokenKind::Lifetime {
            starts_with_number: false,
        }
        | TokenKind::Whitespace
        | TokenKind::Ident
        | TokenKind::RawIdent
        | TokenKind::ForcedKeywordIdent
        | TokenKind::RawLifetime
        | TokenKind::Semi
        | TokenKind::Comma
        | TokenKind::Dot
        | TokenKind::OpenParen
        | TokenKind::CloseParen
        | TokenKind::OpenBrace
        | TokenKind::CloseBrace
        | TokenKind::OpenBracket
        | TokenKind::CloseBracket
        | TokenKind::At
        | TokenKind::Pound
        | TokenKind::Tilde
        | TokenKind::Question
        | TokenKind::Colon
        | TokenKind::Dollar
        | TokenKind::Eq
        | TokenKind::Bang
        | TokenKind::Lt
        | TokenKind::Gt
        | TokenKind::Minus
        | TokenKind::And
        | TokenKind::Or
        | TokenKind::Plus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Caret
        | TokenKind::Percent => Lexeme::Code,
        TokenKind::Eof => Lexeme::End,
    }
}

fn classify_literal(kind: LiteralKind) -> Lexeme {
    match kind {
        LiteralKind::Int {
            empty_int: true, ..
        }
        | LiteralKind::Float {
            empty_exponent: true,
            ..
        } => Lexeme::Malformed(LexErrorKind::MalformedNumber),
        LiteralKind::Char { terminated: false }
        | LiteralKind::Byte { terminated: false }
        | LiteralKind::Str { terminated: false }
        | LiteralKind::ByteStr { terminated: false }
        | LiteralKind::CStr { terminated: false } => {
            Lexeme::Malformed(LexErrorKind::UnterminatedLiteral)
        }
        LiteralKind::RawStr { n_hashes: None }
        | LiteralKind::RawByteStr { n_hashes: None }
        | LiteralKind::RawCStr { n_hashes: None } => {
            Lexeme::Malformed(LexErrorKind::MalformedRawString)
        }
        LiteralKind::Int {
            empty_int: false, ..
        }
        | LiteralKind::Float {
            empty_exponent: false,
            ..
        }
        | LiteralKind::Char { terminated: true }
        | LiteralKind::Byte { terminated: true }
        | LiteralKind::Str { terminated: true }
        | LiteralKind::ByteStr { terminated: true }
        | LiteralKind::CStr { terminated: true }
        | LiteralKind::RawStr { n_hashes: Some(_) }
        | LiteralKind::RawByteStr { n_hashes: Some(_) }
        | LiteralKind::RawCStr { n_hashes: Some(_) } => Lexeme::Code,
    }
}
