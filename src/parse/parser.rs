///  Program for testing and playing with the parser
///

use std::iter::Iterator;
use super::ast::{Query, DefStmt, CreateStmt, DropStmt, CreateTableStmt, AltStmt,
 AlterTableStmt, AlterOp, ColumnInfo, SqlType};               
 use super::token::TokenSpan;
 use super::lex::Lexer;
 use std::mem::swap;
 use super::token::Token;
 use super::Span;

/*+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#Parser public functions
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++*/


// the parser needs a Lexer that iterates through the query
pub struct Parser<'a>{
    lexiter: Lexer<'a>,

    last: Option<TokenSpan>,
    // the current token given by the lexer
    curr: Option<TokenSpan>,
    // next token
    peek: Option<TokenSpan>
}

impl<'a> Parser<'a>{

    /// Constructs a Parser for the given query.
    pub fn create(query: &'a str) -> Parser<'a>{
        let l = Lexer::from_query(query);
        let mut p= Parser{lexiter:l,last:None,curr:None,peek:None};
        // Sets initial position of lexer and curr/peek
        p.lexer_next();
        p.lexer_next();
        p
    }

    /// Parses the given query into an AST
    pub fn parse(&mut self) -> Result<Query, ParseError>{
        //deletes Whitespaces in the beginning of Query
        match self.expect_token(&[Token::Whitespace]){
            Ok(&Token::Whitespace) => self.lexer_next(),
            _=>(),
        }
        // first token is checked if it's a keyword using expect_keyword()
        let querytype = self.expect_keyword(&[Keyword::Create,Keyword::Drop, Keyword::Alter])
        .map_err(|e| match e {
            ParseError::UnexpectedEoq => ParseError::EmptyQueryError,
            _ => e,
        });
        // returns high-level AST or Error
        match try!(querytype) {

            // Different Query-types are matched and returned

            // Create-Query
            Keyword::Create => {
                let query = Query::DefStmt(DefStmt::Create(try!(self.parse_create_stmt())));
                return Ok(try!(self.return_query_ast(query)));
            },
            
            // Alter-Query
            Keyword::Alter => {
                let query = Query::DefStmt(DefStmt::Alter(try!(self.parse_alt_stmt())));
                return Ok(try!(self.return_query_ast(query)));
            },

            // Drop-Query
            Keyword::Drop => {
                let query = Query::DefStmt(DefStmt::Drop(try!(self.parse_drop_stmt())));
                return Ok(try!(self.return_query_ast(query)));
            },

            // Unknown Error
            _=> return Err(ParseError::UnknownError)
        }
    }



/*+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#Parser Functions
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++*/



    // Starts the parsing for tokens in create-syntax
    fn parse_create_stmt(&mut self) -> Result<CreateStmt, ParseError> {
        // Convention: Every method must use skip_whitespace to
        // put the lexer to the position of the token the method needs
        try!(self.skip_whitespace());

        match try!(self.expect_keyword(&[Keyword::Table])) {
            // Create the table subtree
            Keyword::Table=> return return Ok(CreateStmt::Table(try!(self.parse_create_table_stmt()))),
            // Create the view subtree
            // Keyword::View => ...

            // Unknown parsing error
            _=> return Err(ParseError::UnknownError),
        };

    }

    // Parses the tokens fore the create table subtree
    fn parse_create_table_stmt(&mut self) -> Result<CreateTableStmt, ParseError> {
        // Convention: Every method must use skip_whitespace to
        // put the lexer to the position of the token the method needs
        try!(self.skip_whitespace());

        // create a CreateTableStmt Object with the table id
        let mut table_info=CreateTableStmt {tid: try!(self.expect_word()), cols: Vec::<ColumnInfo>::new()};

        self.skip_whitespace();
        // if there is a ParenOp token.....
        if(self.curr.is_none()){
            return Ok(table_info);
        }
        try!(self.expect_token(&[Token::ParenOp]));
        /*match self.expect_token(&[Token::ParenOp]){
            Err(error) => return Ok(table_info),
            Ok(s) => (),
        }*/
        // ...call parse_create_column_vec to generate the column vector subtree
        table_info.cols=try!(self.parse_create_column_vec());
        return Ok(table_info);

    }

    // Parses the tokens for the column vector subtree
    fn parse_create_column_vec(&mut self) -> Result<Vec<ColumnInfo>, ParseError>{
        // Convention: Every method must use skip_whitespace to
        // put the lexer to the position of the token the method needs
        self.skip_whitespace();
        let mut colsvec = Vec::<ColumnInfo>::new();

        // fill the vector with content until ParenCl is the curr token
        while(match self.expect_token(&[Token::ParenCl]){
            Ok(&Token::ParenCl) => false,
            _ => true,
        }){
            // parsing the content for a single ColumnInfo
            colsvec.push(try!(self.parse_column_info()));
            self.skip_whitespace();
            // Check if there is a Comma seperating two columns or a ParenCl ending the vectorparsing
            match try!(self.expect_token(&[Token::Comma, Token::ParenCl])){
                &Token::Comma => {self.skip_whitespace();()},
                _=> (),
            };
        }
        return Ok(colsvec);
    }






    // Parses tokens for alter statement
    fn parse_alt_stmt(&mut self) -> Result<AltStmt, ParseError> {
        try!(self.skip_whitespace());
        
        match try!(self.expect_keyword(&[Keyword::Table])) {
            Keyword::Table=> return Ok(AltStmt::Table(try!(self.parse_alter_table_stmt()))),

            // Unknown parsing error
            _=> return Err(ParseError::UnknownError),
        };
    }


    // Parses table to modify and subsequent operations
    fn parse_alter_table_stmt(&mut self) -> Result<AlterTableStmt, ParseError> {
        try!(self.skip_whitespace());

        let mut alt_table_stmt = AlterTableStmt {tid: try!(self.expect_word()), op: try!(self.parse_alter_op())};
        Ok(alt_table_stmt)

    }

    // Parses operations applied on selected table including tablename and datatype if necessary
    fn parse_alter_op(&mut self) -> Result<AlterOp, ParseError> {
        try!(self.skip_whitespace());

        match try!(self.expect_keyword(&[Keyword::Add, Keyword::Drop, Keyword::Modify])){
            Keyword::Add => return {try!(self.skip_whitespace());Ok(AlterOp::Add(try!(self.parse_column_info())))},
            Keyword::Drop => return {try!(self.skip_whitespace());Ok(AlterOp::Drop(try!(self.expect_word())))},
            Keyword::Modify => return {try!(self.skip_whitespace());Ok(AlterOp::Modify(try!(self.parse_column_info())))},
            _ => return Err(ParseError::UnknownError),
        }
    }

    // Parses the tokens for drop statement
    fn parse_drop_stmt(&mut self) -> Result<DropStmt, ParseError> {
        try!(self.skip_whitespace());

        match try!(self.expect_keyword(&[Keyword::Table])) {
            Keyword::Table => return {try!(self.skip_whitespace()); Ok(DropStmt::Table(try!(self.expect_word())))},
            _=> return Err(ParseError::UnknownError),
        };
    }


/*+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#Utility Functions
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++*/



    // moves current to the next non-whitespace
    fn skip_whitespace(&mut self) -> Result<Token, ParseError>{
        self.lexer_next();
        try!(self.expect_token(&[Token::Whitespace]));
        self.lexer_next();
        return Ok(Token::Whitespace);
    }

     // sets next position for the lexer
     fn lexer_next(&mut self){
        swap(&mut self.last, &mut self.curr);  //  last = curr
        swap(&mut self.curr, &mut self.peek);  //  curr = peek
        self.peek = self.lexiter.next();
    }

    // checks, if query is ended correctly. if yes -> returns query as ast
    fn return_query_ast(&mut self, query: Query) -> Result<Query, ParseError>{
        self.skip_whitespace();
        if(self.curr.is_none()){
            return Ok(query);
        }else{
            return Err(ParseError::InvalidEoq);
        }
    }

        // Utility function to parse metadata of columns
        fn parse_column_info(&mut self) -> Result<ColumnInfo, ParseError> {
        //try!(self.skip_whitespace());   ------- supposed to work, in progress for later, changed l.209-211
        let column_id = try!(self.expect_word());
        try!(self.skip_whitespace());
        let dtype = try!(self.expect_datatype());
        return Ok(ColumnInfo{cid: column_id, datatype: dtype})
    }





        // checks if the current token is a datatype.
    // In case of e.g. char(x) checks if ( ,x and ) are the following
    // token and if x is correct size.
    fn expect_datatype(&mut self) -> Result<SqlType,ParseError> {

        let mut found_datatype;
        let mut span_lo;
        let mut span_hi;
        let tmp_datatype;
        {
            // checks if token non or some
            let token = match self.curr {
                None => return Err(ParseError::UnexpectedEoq),
                // in case of som: return reference to token
                Some(ref token) => token,
            };

            span_lo=token.span.lo;
            span_hi=token.span.hi;

            // checks whether token is a word
            let word = match token.tok {
                Token::Word(ref s) => s,
                _=>return Err(ParseError::NotADatatype(Span {lo: span_lo , hi: span_hi}))
            };
            tmp_datatype = word.to_string();
        }
            // checks if token is a correct Datatype
            found_datatype = match &tmp_datatype[..] {
                "int" => SqlType::Int,
                "bool" => SqlType::Bool,
                "boolean" => SqlType::Bool,
                // checks if char is written in correct sql syntax
                "char" => {
                    self.skip_whitespace();
                    try!(self.expect_token(&[Token::ParenOp]));
                    self.skip_whitespace();
                    let length_string=try!(self.expect_number());
                    self.skip_whitespace();
                    try!(self.expect_token(&[Token::ParenCl]));
                    let length = match length_string.parse::<u8>() {
                        Ok(length) => length,
                        Err(error) => return Err(ParseError::DatatypeMissmatch(Span {lo: span_lo , hi: span_hi})),
                    };
                    ;SqlType::Char(length)
                },


                // checks if char is written in correct sql syntax
                "varchar" => {
                    self.skip_whitespace();
                    try!(self.expect_token(&[Token::ParenOp]));
                    self.skip_whitespace();
                    let length_string=try!(self.expect_number());
                    self.skip_whitespace();
                    try!(self.expect_token(&[Token::ParenCl]));
                    let length = match length_string.parse::<u16>() {
                        Ok(length) => length,
                        Err(error) => return Err(ParseError::DatatypeMissmatch(Span {lo: span_lo , hi: span_hi})),
                    };
                    ;SqlType::VarChar(length)
                },

                _=>return Err(ParseError::NotADatatype(Span {lo: span_lo , hi: span_hi})),

            };

            return Ok((found_datatype));

        }


    // checks if the current token is a word
    fn expect_word(&self) -> Result<String, ParseError>{
        let mut found_word;
        let mut span_lo;
        let mut span_hi;
        {
            // checks if token non or some
            let token = match self.curr {
                None => return Err(ParseError::UnexpectedEoq),
                // in case of som: return reference to token
                Some(ref token) => token,
            };

            span_lo=token.span.lo;
            span_hi=token.span.hi;

            // checks whether token is a word
            found_word = match token.tok {
                Token::Word(ref s) => s,
                _=>return Err(ParseError::NotAWord(Span {lo: span_lo , hi: span_hi}))
            };

        }
        return Ok(found_word.to_string());


    }

    // checks if the current token is a number
    fn expect_number(&self) -> Result<String, ParseError>{
        let mut found_num;
        let mut span_lo;
        let mut span_hi;
        {
            // checks if token non or some
            let token = match self.curr {
                None => return Err(ParseError::UnexpectedEoq),
                // in case of som: return reference to token
                Some(ref token) => token,
            };

            span_lo=token.span.lo;
            span_hi=token.span.hi;

            // checks whether token is a valid number
            found_num = match token.tok {
                Token::Num(ref s) => s,
                _=>return Err(ParseError::NotANumber(Span {lo: span_lo , hi: span_hi}))
            };

        }
        return Ok(found_num.to_string());
    }

    // checks if current token is an expected token
    fn expect_token(& self,expected_tokens: &[Token]) -> Result<&Token, ParseError>{

            // checks if current is none or some
            let token = match self.curr {
                None => return Err(ParseError::UnexpectedEoq),
                // in case of some: return reference to token
                Some(ref token) => token,
            };


            if(expected_tokens.contains(&(token.tok))){
                return Ok(&token.tok);
            }else{
                return Err(ParseError::WrongToken(Span {lo: token.span.lo, hi: token.span.hi}))
            }
        }

    // matches current token against any keyword and checks if it is one of the expected keywords
    fn expect_keyword(&self,expected_keywords: &[Keyword]) -> Result<Keyword, ParseError> {
        let mut found_keyword;
        let mut span_lo;
        let mut span_hi;
        {
            // checks if token non or some
            let token = match self.curr {
                None => return Err(ParseError::UnexpectedEoq),
                // in case of som: return reference to token
                Some(ref token) => token,
            };

            span_lo=token.span.lo;
            span_hi=token.span.hi;

            // checks whether token is a word
            let word = match token.tok {
                Token::Word(ref s) => s,
                _=>return Err(ParseError::NotAKeyword(Span {lo: span_lo , hi: span_hi}))
            };

            // checks if word is a keyword
            found_keyword = match &word[..] {
                "create" => Keyword::Create,
                "drop" => Keyword::Drop,
                "table" => Keyword::Table,
                "view" => Keyword::View,
                "alter" => Keyword::Alter,
                "update" => Keyword::Update,
                "select" => Keyword::Select,
                "insert" => Keyword::Insert,
                "delete" => Keyword::Delete,
                "modify" => Keyword::Modify,
                "add" => Keyword::Add,
                "column" => Keyword::Column,
                _=>return Err(ParseError::NotAKeyword(Span {lo: span_lo , hi: span_hi})),

            };
        }
        // checks if keyword is expected keyword
        if(expected_keywords.contains(&found_keyword)){
            return Ok((found_keyword));
        }else{
            return Err(ParseError::WrongKeyword(Span {lo: span_lo , hi: span_hi}));
        }
    }
}


/*+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#Enums
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++*/



                    #[derive(PartialEq)]
// Keywords that can occour in SQL syntax
pub enum Keyword {
    // 1st level keywords
    // data definition keywords
    Create,
    Drop,
    Alter,

    // data manipulation keywords
    Select,
    Update,
    Insert,
    Delete,

    // 2nd level keywords
    Table,
    View,
    Column,

    // 3rd level keywords
    From,
    Where,
    Modify,
    Add
}

#[derive(Debug)]
pub enum ParseError {
    //general errors
    UnknownError,
    EmptyQueryError,

    // Syntax errors:
    //End of file,
    UnexpectedEoq,
    InvalidEoq,

    //Token errors
    WrongKeyword(Span),
    WrongToken(Span),
    DatatypeMissmatch(Span),
    NotAKeyword(Span),
    NotAToken(Span),
    NotAWord(Span),
    NotADatatype(Span),
    NotANumber(Span),




    //Used for debugging
    DebugError(String)
// TODO: introduce good errors and think more about it
}


