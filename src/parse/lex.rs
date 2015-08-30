use super::token::{Token, TokenSpan};
use super::Span;

pub struct Lexer<'a> {
    query: &'a str,
    curr: Option<char>,
    curr_pos: Option<u32>,
}

impl<'a> Lexer<'a> {
    pub fn from_query<'b>(query: &'b str) -> Lexer<'b> {
        Lexer {
            query: query,
            curr: None,
            curr_pos: None
        }
    }

    pub fn next(&mut self) -> Option<TokenSpan> {
        None
    }
}