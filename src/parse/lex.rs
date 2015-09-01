use super::token::{Token, TokenSpan};
use std::str::Chars;
use super::Span;
use std::iter::{Iterator};

///A lexer with its associated query, a char iterator, and
///positions (last, current, next)
pub struct Lexer<'a> {
    query: &'a str,
    chs : Chars<'a>,
    last: Option<char>,
    last_pos: Option<usize>,
    curr: Option<char>,
    curr_pos: Option<usize>,
    next: Option<char>
}

impl<'a> Lexer<'a> {

    ///new lexer, everything set to None
    pub fn from_query<'b>(query: &'b str) -> Lexer<'b> {
        let mut lex = Lexer {
            query: query,
            curr: None,
            curr_pos: Some(0),
            last: None,
            last_pos: Some(0),
            next: None,
            chs : query.chars()
        };
        lex.dbump();
        lex
    }

    ///bumper function advances to the next char
    fn bump(&mut self){

        //advance all pointers to the next char
        self.last = self.curr;
        self.curr = self.next;
        self.next = self.chs.next();


        //TODO: position-management
        //advance last_pos to position of current char
        // self.last_pos = self.curr_pos;

        //next position is current position plus the utf8 length
        //of the next character
        // match self.next {
        //     Some(c) => {
        //         self.curr_pos = Some(self.curr_pos.unwrap_or("err")
        //             + c.len_utf8())
        //     }
        //     _ => {}
        // };

    }

    ///double bump
    fn dbump(&mut self) {
        self.bump();
        self.bump();
    }

    ///scan each new token from the query string
    fn scan_token(&mut self) -> String {
        let mut s = String::new();
        //loop until the end of a word (only letters)
        loop {
            //take current char
            match self.curr.unwrap_or(' ') {
                c @ 'a' ... 'z' |
                c @ 'A' ... 'Z' => {
                    //push letter into return string
                    s.push(c);
                },
                //if no letter, then stop loop
                _ => break
            }
            //next char
            self.bump();
        }
        //:'(
        s
    }


    ///skips all the whitespaces
    fn skip_whitespace(&mut self) {
        while is_whitespace(self.curr.unwrap()){
            self.bump();
        }
    }
}

///checks for whitespace/line break/tab
fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\n' | '\t' => true,
        _ => false
    }
}

impl<'a> Iterator for Lexer<'a> {

    type Item = TokenSpan;

    fn next(&mut self) -> Option<TokenSpan> {

        //if no current char available, return None
        if self.curr.is_none() {
            return None;
        }

        //unwrap current char and decide token class
        let token = match self.curr.unwrap_or('x') {

            //whitespaces
            c if is_whitespace(c) => {
                self.skip_whitespace();
                Token::Whitespace
            },

            //exclamation marks
            '!' => {
                self.bump();
                Token::Bang
            },

            //question marks
            '?' => {
                self.bump();
                Token::QMark
            },

            //dots
            '.' => {
                self.bump();
                Token::Dot
            },

            //commas
            ',' => {
                self.bump();
                Token::Comma
            },

            //words
            'a' ... 'z' | 'A' ... 'Z' => {
                let w = self.scan_token();
                Token::Word
            },

            //default: everything else
            _ => {
                self.bump();
                Token::Unknown
            }

        };

        //return Option
        Some(TokenSpan {
            tok: token,
            //currently 0 TODO
            span: Span { lo: 0, hi: 0}
        })
    }
}
