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
    next: Option<char>,
    span_start: Option<usize>
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
            span_start: Some(0),
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
        // advance last_pos to position of current char
        self.last_pos = self.curr_pos;

        // next position is current position plus the utf8 length
        // of the next character
        match self.next {
            Some(c) => {
                self.curr_pos = Some(self.curr_pos.unwrap()
                    + c.len_utf8())
            }
            _ => {}
        };

    }

    ///double bump
    fn dbump(&mut self) {
        self.bump();
        self.bump();
    }

    ///scan each new WORD from the query string
    fn scan_words(&mut self) -> String {
        let mut s = String::new();
        //loop until the end of a word (only letters and _)
        loop {
            //take current char
            match self.curr.unwrap_or(' ') {
                c @ 'a' ... 'z' |
                c @ 'A' ... 'Z' |
                c @ '0' ... '9' |
                c @ '_' => {
                    //push letter into return string
                    s.push(c);
                },
                //if no letter, then stop loop
                _ => break
            }
            //next char
            self.bump();
        }
        s
    }

    ///scan each new NUMBER from the query string
    fn scan_nums(&mut self) -> String {
        let mut s = String::new();
        let mut dot = false;
        loop {
            match self.curr.unwrap_or(' ') {
                c @ '0' ... '9' => {
                    s.push(c);
                }
                c @ '.' => {
                    if dot {
                        println!("This is not a good number, Sir!");
                    }
                    dot = true;
                    s.push(c);
                },
                _ => break
            }
            self.bump();
        }
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

        //for two char ops (e.g. >=), we need to check the next char
        //if we are at the end of the string, we end
        let nexchar = self.next.unwrap_or('\x00');

        //saving the first pos of a token
        self.span_start = self.curr_pos;

        //if no current char available, return None
        if self.curr.is_none() {
            return None;
        }

        //unwrap current char and decide token class
        let token = match self.curr.unwrap_or('x') {

            //words
            'a' ... 'z' | 'A' ... 'Z' => {
                let w = self.scan_words();
                Token::Word(w)
            },

            //nums
            '0' ... '9' => {
                let n = self.scan_nums();
                Token::Num(n)

            },

            //semi
            ';' => {
                self.bump();
                Token::Semi
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

            //ParenOp
            '(' => {
                self.bump();
                Token::ParenOp
            },

            //ParenCl
            ')' => {
                self.bump();
                Token::ParenCl
            },

            //ADel
            '\'' | '"' => {
                self.bump();
                Token::ADel
            },

            //Equ
            '=' => {
                self.bump();
                Token::Equ
            },

            //GEThan >=
            '>' if nexchar == '=' => {
                self.dbump(); //because two chars!
                Token::GEThan
            },

            //SEThan <=
            '<' if nexchar == '=' => {
                self.dbump();
                Token::SEThan
            },

            //GThan >
            '>' => {
                self.bump();
                Token::GThan
            },

            //SThan <
            '<' => {
                self.bump();
                Token::SThan
            },

            //NEqu !=
            '!' if nexchar == '=' => {
                self.dbump();
                Token::NEqu
            },

            //Add
            '+' => {
                self.bump();
                Token::Add
            },

            //Sub
            '-' => {
                self.bump();
                Token::Sub
            },

            //Div
            '/' => {
                self.bump();
                Token::Div
            }

            //Star
            '*' => {
                self.bump();
                Token::Star
            },

            //whitespaces
            c if is_whitespace(c) => {
                self.skip_whitespace();
                Token::Whitespace
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
            span: Span { lo: self.span_start.unwrap(), hi: self.curr_pos.unwrap()}
        })
    }
}
