use super::Span;
use parse::ast::*;
/// A token with it's associated Span in the source code
#[derive(Debug)]
pub struct TokenSpan {
    pub tok: Token,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
	String(String),
	Int(i64),
    Float(f64),
    Bool(u8),
}

impl Lit {
    pub fn into_DataSrc(&self) -> DataSrc {
        match self {
            &Lit::String(ref s) => DataSrc::String(s.clone()),
            &Lit::Int(ref i) => DataSrc::Int(i.clone()),
            &Lit::Float(ref f) => DataSrc::String(f.to_string()),
            &Lit::Bool(ref b) => DataSrc::Bool(b.clone()),
        }
    }
}

/// A token: Everything the lexer can produce
#[derive(Debug, Clone, PartialEq)]
pub enum Token {

    Word(String),

    // detects literals
    Literal(Lit),

    Semi,
    Dot,
    Comma,
    // Bang,
    // QMark,

    // delimiter (,),',"
    ParenOp,
    ParenCl,
    ADel,

    // mathematic ops
    Equ,
    GThan,
    SThan,
    GEThan,
    SEThan,
    NEqu,
    Add,
    Sub,
    Div,
    Mod,

    // sensitive Wildcard/Mult, eval in parser
    Star,

    Whitespace,

    Unknown
}
