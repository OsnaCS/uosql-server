use super::token::{Token, TokenSpan, Lit};
use std::str::Chars;
use super::Span;
use std::iter::{Iterator};

/// A lexer with its associated query, a char iterator, and
/// positions (last, current, next)
pub struct Lexer<'a> {
    chs : Chars<'a>,
    last: Option<char>,
    last_pos: Option<usize>,
    curr: Option<char>,
    curr_pos: Option<usize>,
    next: Option<char>,
    span_start: Option<usize>
}

impl<'a> Lexer<'a> {

    /// New lexer, everything set to None
    pub fn from_query<'b>(query: &'b str) -> Lexer<'b> {
        let mut lex = Lexer {
            curr: None,
            curr_pos: None,
            last: None,
            last_pos: None,
            next: None,
            span_start: None,
            chs : query.chars()
        };
        lex.dbump();
        lex
    }

    /// Bumper function advances to the next char
    fn bump(&mut self){

        // advance all pointers to the next char
        self.last = self.curr;
        self.curr = self.next;
        self.next = self.chs.next();


        // Advance last_pos to position of current char
        self.last_pos = self.curr_pos;

        // Next position is current position plus the utf8 length
        // of the next character
        match self.next {
            Some(c) => {
                self.curr_pos = match self.curr_pos {
                    Some(n) => Some(n + c.len_utf8()),
                    None => Some(0 + c.len_utf8()) // Start at pos 0
                }
            }
            _ => {}
        };

    }

    /// Double bump
    fn dbump(&mut self) {
        self.bump();
        self.bump();
    }

    /// Scan each new WORD from the query string
    fn scan_words(&mut self) -> String {
        let mut s = String::new();
        // Loop until the end of a word (only letters, numbers and _)
        loop {
            // Take current char
            match self.curr.unwrap_or(' ') {
                c @ 'a' ... 'z' |
                c @ 'A' ... 'Z' |
                c @ '0' ... '9' |
                c @ '_' => {
                    // Push letter into return string
                    s.push(c);
                },
                // If no letter, then stop loop
                _ => break
            }
            // Next char
            self.bump();
        }
        s
    }

    /// Scan each new NUMBER from the query string
    fn scan_nums(&mut self) -> String {
        let mut s = String::new();
        let mut dot = false;
        loop {
            match self.curr.unwrap_or(' ') {
                c @ '0' ... '9' |
                c @ '.' => {
                    s.push(c);
                }
                _ => break
            }
            self.bump();
        }
        s
    }

    fn scan_lit(&mut self) -> String {
        let mut s = String::new();
        self.bump(); // To first char of literal
        loop {
            match self.curr.unwrap_or(' ') {
                c @ '\'' |
                c @ '"' => {
                    break
                }
                c @ _ => {
                    s.push(c);
                }
            }
            self.bump();
        }
        s
    }

    /// Skips all the whitespaces
    fn skip_whitespace(&mut self) {
        while is_whitespace(self.curr.unwrap_or('x')){
            self.bump();
        }
    }
}

/// Checks for whitespace/line break/tab
fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\n' | '\t' => true,
        _ => false
    }
}

impl<'a> Iterator for Lexer<'a> {

    type Item = TokenSpan;

    fn next(&mut self) -> Option<TokenSpan> {

        // For two char ops (e.g. >=), we need to check the next char
        // if we are at the end of the string, we end
        let nexchar = self.next.unwrap_or('\x00');

        // Saving the first pos of a token
        self.span_start = self.curr_pos;

        // Getting current char, else return None
        let curr = match self.curr {
            None => return None,
            Some(c) => c
        };

        // Matching current char to respective token
        let token = match curr {

            // Words
            'a' ... 'z' | 'A' ... 'Z' => {
                let w = self.scan_words();
                Token::Word(w.to_lowercase())
            },

            // Nums
            '0' ... '9' => {
                let n = self.scan_nums();
                Token::Num(n)

            },

            // Semicolon
            ';' => {
                self.bump();
                Token::Semi
            },

            // Dots
            '.' => {
                self.bump();
                Token::Dot
            },

            // Commas
            ',' => {
                self.bump();
                Token::Comma
            },

            // //bang
            // '!' => {
            //     self.bump();
            //     Token::Bang
            // },

            // //question marks
            // '?' => {
            //     self.bump();
            //     Token::QMark
            // },

            // ParenOp
            '(' => {
                self.bump();
                Token::ParenOp
            },

            // ParenCl
            ')' => {
                self.bump();
                Token::ParenCl
            },

            // Literals
            '\'' | '"' => {
                let l = self.scan_lit();
                self.bump();
                Token::Literal(Lit::Str(l))
            },

            // Equ
            '=' => {
                self.bump();
                Token::Equ
            },

            // GEThan >=
            '>' if nexchar == '=' => {
                self.dbump(); //because two chars!
                Token::GEThan
            },

            // SEThan <=
            '<' if nexchar == '=' => {
                self.dbump();
                Token::SEThan
            },

            // GThan >
            '>' => {
                self.bump();
                Token::GThan
            },

            // NEqu <>
            '<' if nexchar == '>' => {
                self.dbump();
                Token::NEqu
            },

            // SThan <
            '<' => {
                self.bump();
                Token::SThan
            },

            // Add
            '+' => {
                self.bump();
                Token::Add
            },

            // Sub
            '-' => {
                self.bump();
                Token::Sub
            },

            // Div
            '/' => {
                self.bump();
                Token::Div
            }

            // Mod
            '%' => {
                self.bump();
                Token::Mod
            }

            // Star
            '*' => {
                self.bump();
                Token::Star
            },

            // Whitespaces
            c if is_whitespace(c) => {
                self.skip_whitespace();
                Token::Whitespace
            },

            // Default: everything else
            _ => {
                self.bump();
                Token::Unknown
            }

        };

        // Return an Option
        Some(TokenSpan {
            tok: token,
            span: Span { lo: self.span_start.unwrap(), hi: self.curr_pos.unwrap()}
        })
    }
}
