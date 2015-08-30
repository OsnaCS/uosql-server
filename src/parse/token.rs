use super::Span;


/// A token with it's associated Span in the source code
pub struct TokenSpan {
    tok: Token,
    span: Span,
}

/// A token: Everything the lexer can produce
pub enum Token {
    Semi,
    Unknown
}