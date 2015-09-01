use super::Span;

/// A token with it's associated Span in the source code
#[derive(Debug)]
pub struct TokenSpan {
    pub tok: Token,
    pub span: Span,
}

/// A token: Everything the lexer can produce
#[derive(Debug)]
pub enum Token {
    Word,
    Whitespace,
    Bang,
    Dot,
    QMark,
    Comma,
    Unknown
}
