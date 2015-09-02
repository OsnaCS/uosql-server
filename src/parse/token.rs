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

    Word(String),
    Num(String),

    Semi,
    Dot,
    Comma,
    // Bang,
    // QMark,

    //delimiter (,),',"
    ParenOp,
    ParenCl,
    ADel,

    //mathematic ops
    Equ,
    GThan,
    SThan,
    GEThan,
    SEThan,
    NEqu,
    Add,
    Sub,
    Div,

    //sensitive Wildcard/Mult, eval in parser
    Star,

    Whitespace,

    Unknown
}
