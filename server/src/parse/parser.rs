///  Program for testing and playing with the parser
///

use std::iter::Iterator;
use super::ast::*;
use super::token::{TokenSpan, Lit};
use super::lex::Lexer;
use std::mem::swap;
use super::token::Token;
use super::Span;
use super::super::storage::SqlType;
use std::collections::HashMap;
use super::lex;

// ===========================================================================
// Parser public functions
// ===========================================================================


// the parser needs a Lexer that iterates through the query
pub struct Parser<'a> {
    lexiter: Lexer<'a>,

    last: Option<TokenSpan>,
    // the current token given by the lexer
    curr: Option<TokenSpan>,
    // next token
    peek: Option<TokenSpan>
}

impl<'a> Parser<'a> {

    /// Constructs a Parser for the given query.
    pub fn create(query: &'a str) -> Parser<'a> {
        let l = Lexer::from_query(query);
        let mut p = Parser { lexiter: l, last: None, curr: None, peek: None };
        // Sets initial position of lexer and curr/peek
        p.bump();
        p.bump();
        p
    }

    /// Parses the given query into an AST
    pub fn parse(&mut self) -> Result<Query, ParseError> {
        // deletes Whitespaces in the beginning of Query

        // first token is checked if it's a keyword using expect_keyword()
        let keywords = &[Keyword::Create, Keyword::Drop, Keyword::Alter,
        Keyword::Use, Keyword::Delete, Keyword::Insert, Keyword::Describe,
        Keyword::Update, Keyword::Select];
        let querytype = self.expect_keyword(keywords).map_err(|e| match e {
            ParseError::UnexpectedEoq => ParseError::EmptyQueryError,
            _ => e,
        });

        match try!(querytype) {
            // Create-Query
            Keyword::Create => {
                let query = Query::DefStmt(DefStmt::Create(try!(self.parse_create_stmt())));
                Ok(try!(self.return_query_ast(query)))
            },
            // Alter-Query
            Keyword::Alter => {
                let query = Query::DefStmt(DefStmt::Alter(
                    try!(self.parse_alt_stmt())
                    ));
                Ok(try!(self.return_query_ast(query)))
            },
            // Drop-Query
            Keyword::Drop => {
                let query = Query::DefStmt(DefStmt::Drop(
                    try!(self.parse_drop_stmt())
                    ));
                Ok(try!(self.return_query_ast(query)))
            },
            // Use-Query
            Keyword::Use => {
                let query = Query::ManipulationStmt(ManipulationStmt::Use(
                    try!(self.parse_use_stmt())
                    ));
                Ok(try!(self.return_query_ast(query)))
            }
            // Insert-Query
            Keyword::Insert => {
                let query = Query::ManipulationStmt(ManipulationStmt::Insert(
                    try!(self.parse_insert_stmt())
                    ));
                Ok(try!(self.return_query_ast(query)))
            }
            //Update-Query
            Keyword::Update => {
                let query = Query::ManipulationStmt(ManipulationStmt::Update(
                    try!(self.parse_update_stmt())
                    ));
                Ok(try!(self.return_query_ast(query)))
            },
            // Delete-Query
            Keyword::Delete => {
                let query = Query::ManipulationStmt(ManipulationStmt::Delete(
                    try!(self.parse_delete_stmt())
                    ));
                Ok(try!(self.return_query_ast(query)))
            }
            //Describe-Query
            Keyword::Describe => {
                try!(self.bump());
                let query = Query::ManipulationStmt(ManipulationStmt::Describe(
                    try!(self.expect_word(false))
                    ));
                Ok(try!(self.return_query_ast(query)))
            }
            //Select-Query
            Keyword::Select => {
                let query = Query::ManipulationStmt(ManipulationStmt::Select(
                    try!(self.parse_select_stmt())
                    ));
                Ok(try!(self.return_query_ast(query)))
            }

            // Unknown Error
            _ => Err(ParseError::UnknownError)
        }
    }




// =============================================================================
// Parser Functions
// =============================================================================


    // Starts the parsing for tokens in create-syntax
    fn parse_create_stmt(&mut self) -> Result<CreateStmt, ParseError> {
        // Convention: Every method must use bump to
        // put the lexer to the position of the token the method needs
        try!(self.bump());
        let mut view_check = false;
        if self.expect_keyword(&[Keyword::Or]).is_ok() {
            try!(self.bump());
            try!(self.expect_keyword(&[Keyword::Replace]));
            try!(self.bump());
            view_check = true;
        }

        match try!(self.expect_keyword(&[Keyword::Table, Keyword::Database, Keyword::View,
            Keyword::Or])) {
            // Create the table subtree
            Keyword::Table => Ok(CreateStmt::Table(try!(self.parse_create_table_stmt()))),
            // Create Database subtree
            Keyword::Database => {
                try!(self.bump());
                Ok(CreateStmt::Database(try!(self.expect_word(false))))
            },
            // Create View subtree
            Keyword::View => {
                try!(self.bump());
                let name = try!(self.expect_word(false));
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::As]));
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Select]));
                Ok(CreateStmt::View(CreateViewStmt {
                    name: name,
                    opt: view_check,
                    sel: try!(self.parse_select_stmt()),
                }))
            },
            _ => Err(ParseError::UnknownError),
        }
    }

    // Parses the tokens fore the create table subtree
    fn parse_create_table_stmt(&mut self) -> Result<CreateTableStmt, ParseError> {
        // Convention: Every method must use bump to
        // put the lexer to the position of the token the method needs
        try!(self.bump());

        // create a CreateTableStmt Object with the table id
        let mut table_info = CreateTableStmt {
            tid: try!(self.expect_word(false)),
            cols: Vec::<ColumnInfo>::new()
        };
        try!(self.bump());
        // if there is a ParenOp token.....
        if self.curr.is_none() {
            return Ok(table_info)
        }
        try!(self.expect_token(&[Token::ParenOp]));
        // ...call parse_create_column_vec to generate the column vector subtree
        table_info.cols = try!(self.parse_create_column_vec());
        Ok(table_info)
    }

    // Parses the tokens for the column vector subtree
    fn parse_create_column_vec(&mut self) -> Result<Vec<ColumnInfo>, ParseError> {
        // Convention: Every method must use bump to
        // put the lexer to the position of the token the method needs
        try!(self.bump());
        let mut colsvec = Vec::<ColumnInfo>::new();

        // fill the vector with content until ParenCl is the curr token
        while !self.expect_token(&[Token::ParenCl]).is_ok()
        {
            // parsing the content for a single ColumnInfo
            colsvec.push(try!(self.expect_column_info()));
            try!(self.bump());
            // Check if there is a Comma seperating two columns or a ParenCl
            // ending the vectorparsing
            match try!(self.expect_token(&[Token::Comma, Token::ParenCl])) {
                Token::Comma => try!(self.bump()),
                _ => (),
            };
        }
        Ok(colsvec)
    }

    // Parses tokens for alter statement
    fn parse_alt_stmt(&mut self) -> Result<AltStmt, ParseError> {
        try!(self.bump());
        match try!(self.expect_keyword(&[Keyword::Table])) {
            Keyword::Table => Ok(AltStmt::Table(try!(self.parse_alter_table_stmt()))),

            // Unknown parsing error
            _ => Err(ParseError::UnknownError),
        }
    }

    // Parses table to modify and subsequent operations
    fn parse_alter_table_stmt(&mut self) -> Result<AlterTableStmt, ParseError> {
        try!(self.bump());
        let alt_table_stmt = AlterTableStmt {
            tid: try!(self.expect_word(false)),
            op: try!(self.parse_alter_op())
        };
        Ok(alt_table_stmt)
    }

    // Parses operations applied on selected table including tablename and
    // datatype if necessary
    fn parse_alter_op(&mut self) -> Result<AlterOp, ParseError> {
        try!(self.bump());
        match try!(self.expect_keyword(&[Keyword::Add, Keyword::Drop, Keyword::Modify])) {
            Keyword::Add => {
                try!(self.bump());
                Ok(AlterOp::Add(try!(self.expect_column_info())))
            },
            Keyword::Drop => {
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Column]));
                try!(self.bump());
                Ok(AlterOp::Drop(try!(self.expect_word(true))))
            },
            Keyword::Modify => {
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Column]));
                try!(self.bump());
                Ok(AlterOp::Modify(try!(self.expect_column_info())))
            },
            _ => Err(ParseError::UnknownError),
        }
    }

    // Parses the tokens for drop statement
    fn parse_drop_stmt(&mut self) -> Result<DropStmt, ParseError> {
        try!(self.bump());
        match try!(self.expect_keyword(&[Keyword::Table, Keyword::Database, Keyword::View])) {
            Keyword::Table => {
                try!(self.bump());
                Ok(DropStmt::Table(try!(self.expect_word(false))))
            },
            Keyword::Database => {
                try!(self.bump());
                Ok(DropStmt::Database(try!(self.expect_word(false))))
            },
            Keyword::View => {
                try!(self.bump());
                Ok(DropStmt::View(try!(self.expect_word(false))))
            },
            _ => Err(ParseError::UnknownError),
        }
    }

    // Parses the tokens for use statement
    fn parse_use_stmt(&mut self) -> Result<UseStmt, ParseError> {
        try!(self.bump());
        match try!(self.expect_keyword(&[Keyword::Database])) {
            Keyword::Database => {
                try!(self.bump());
                Ok(UseStmt::Database(try!(self.expect_word(false))))
            },
            _ => Err(ParseError::UnknownError),
        }
    }

    // Parses tokens for insert statement
    fn parse_insert_stmt(&mut self) -> Result<InsertStmt, ParseError> {
        try!(self.bump());
        match try!(self.expect_keyword(&[Keyword::Into])) {
            Keyword::Into => (),
            _ => return Err(ParseError::UnknownError),
        }

        try!(self.bump());
        let i = InsertStmt {
            tid: try!(self.expect_word(false)),
            col: try!(self.parse_insert_stmt_detail()),
            val: try!(self.parse_insert_stmt_value()),
        };

        if i.col.len() != 0 && i.col.len() != i.val.len() {
            return Err(ParseError::ColumnCountMissmatch);
        }
        Ok(i)
    }

    // Parses columns for insert statement
    fn parse_insert_stmt_detail(&mut self) -> Result<Vec<String>, ParseError> {
        try!(self.bump());

        let mut res_vec = Vec::<String>::new();

        if self.expect_token(&[Token::ParenOp]).is_ok() {
            // includes additional bump due to closing par
            res_vec = try!(self.parse_insert_stmt_column());
        } else {
            return Ok(res_vec);
        }
        Ok(res_vec)
    }

    // Continuation of parse_insert_stmt_detail
    fn parse_insert_stmt_column(&mut self) -> Result<Vec<String>, ParseError> {
        try!(self.bump());

        let mut res_vec = Vec::<String>::new();

        // fill the vector with content until ParenCl is the curr token
        while !self.expect_token(&[Token::ParenCl]).is_ok() {
            // parsing colums that will function as insert target
            res_vec.push(try!(self.expect_word(true)));
            try!(self.bump());
            // Check if there is a Comma seperating two columns or a ParenCl
            // ending the vectorparsing
            match try!(self.expect_token(&[Token::Comma, Token::ParenCl])) {
                Token::Comma => try!(self.bump()),
                _ => (),
            };
        }
        try!(self.bump());
        Ok(res_vec)
    }

    // Parses i.val of parse_insert_stmt
    fn parse_insert_stmt_value(&mut self) -> Result<Vec<Lit>, ParseError> {
        let mut res_vec = Vec::<Lit>::new();
        match try!(self.expect_keyword(&[Keyword::Values])) {
            Keyword::Values => (),
            _ => return Err(ParseError::UnknownError),
        }
        try!(self.bump());
        try!(self.expect_token(&[Token::ParenOp]));
        try!(self.bump());
        // fill the vector with content until ParenCl is the curr token
        while !self.expect_token(&[Token::ParenCl]).is_ok() {
            // parsing the content for a single column
            let lit = try!(self.expect_literal());

            res_vec.push(lit);
            try!(self.bump());
            // Check if there is a Comma seperating two columns or a ParenCl
            // ending the vectorparsing
            match try!(self.expect_token(&[Token::Comma, Token::ParenCl])) {
                Token::Comma => try!(self.bump()),
                _ => (),
            };
        }
        Ok(res_vec)
    }

    // parses update - query
    fn parse_update_stmt(&mut self) -> Result<UpdateStmt, ParseError> {
        //parsing the name of the table and checking update x set syntax
        try!(self.bump());
        let tableid = try!(self.expect_word(false));
        let mut aliasmap = HashMap::new();
        if !self.check_next_keyword(&[Keyword::Set]){
            try!(self.bump());
            aliasmap.insert(try!(self.expect_word(false)), tableid.clone());
        }
        try!(self.bump());
        try!(self.expect_keyword(&[Keyword::Set]));
        let mut setvec = Vec::new();
        let mut done = false;
        //parsing optional update changes, at least one
        while !done
        {
            try!(self.bump());
            //parse optional alias
            let mut alias = None;
            if self.check_next_token(&[Token::Dot]) {
                alias = Some(try!(self.expect_word(false)));
                try!(self.bump());
                try!(self.bump());
            };
            let column = try!(self.expect_word(true));
            try!(self.bump());
            try!(self.expect_token(&[Token::Equ]));
            try!(self.bump());
            let value = try!(self.expect_literal());
            try!(self.bump());
            setvec.push(Condition {
                aliascol: alias,
                col: column,
                op: CompType::Equ,
                aliasrhs: None,
                rhs: CondType::Literal(value)
            } );
            if !self.expect_token(&[Token::Comma]).is_ok() {
                done = true;
            }
        }
        Ok(UpdateStmt { tid: tableid, alias: aliasmap, set: setvec, conds:
                if self.expect_keyword(&[Keyword::Where]).is_ok() {
                    Some(try!(self.parse_where_part()))
                } else {
                    None
                }
            }
        )
    }

    // Parses the tokens for delete statement
    fn parse_delete_stmt(&mut self) -> Result<DeleteStmt, ParseError> {
        try!(self.bump());
        try!(self.expect_keyword(&[Keyword::From]));
        try!(self.bump());
        let tableid = try!(self.expect_word(false));
        let mut aliasmap = HashMap::new();
        if !self.check_next_keyword(&[Keyword::Where]) {
            try!(self.bump());
            match self.expect_word(false) {
                Err(ParseError::UnexpectedEoq) => (),
                Err(err) => return Err(err),
                Ok(s) => { aliasmap.insert(s.clone(), tableid.clone()); () },
            }
        }
        try!(self.bump());
        let conditiontree = match self.expect_keyword(&[Keyword::Where]) {
            Ok(Keyword::Where) => Some(try!(self.parse_where_part())),
            _ => None,
        };
        Ok(DeleteStmt { tid: tableid, alias: aliasmap, cond: conditiontree } )
    }

    // Parses the tokens for select statement
    fn parse_select_stmt(&mut self) -> Result<SelectStmt, ParseError>{
        let mut targetvec = Vec::new();
        let mut done = false;
        // parsing optional targets, at least one
        while !done
        {
            // optional table alias
            try!(self.bump());
            let mut targetalias = None;
            if self.check_next_token(&[Token::Dot]) {
                targetalias = Some(try!(self.expect_word(false)));
                try!(self.bump());
                try!(self.bump());
            };
            // required target column
            let targetcol = match self.expect_token(&[Token::Star]) {
                Err(err) => Col::Specified(try!(self.expect_word(true))),
                Ok(Token::Star) => Col::Every,
                _ => return Err(ParseError::UnknownError) ,
                };
            try!(self.bump());
            // optional target column rename
            let mut targetrename = None;
            if self.expect_keyword(&[Keyword::As]).is_ok() {
                try!(self.bump());
                targetrename = Some(try!(self.expect_word(true)));
                try!(self.bump());
            }
            targetvec.push(Target { alias: targetalias, col: targetcol, rename: targetrename} );

            if !self.expect_token(&[Token::Comma]).is_ok() {
                done = true;
            }
        }
        // parsing the from list, at least one table required
        try!(self.expect_keyword(&[Keyword::From]));
        let mut tidvec = Vec::new();
        let mut aliasmap = HashMap::new();
        done = false;
        // parsing optional tables
        while !done
        {
            try!(self.bump());
            let tableid = try!(self.expect_word(false));
            if !self.check_next_keyword(&[Keyword::Where, Keyword::Limit,
                Keyword::Group, Keyword::Order])
            && !self.check_next_token(&[Token::Comma]) {
                try!(self.bump());
                match self.expect_word(false) {
                    Err(ParseError::UnexpectedEoq) => (),
                    Err(err) => return Err(err),
                    Ok(s) => { aliasmap.insert(s.clone(), tableid.clone()); () },
                }
            }
            tidvec.push(tableid);
            if !self.check_next_token(&[Token::Comma]) {
                done = true;
                try!(self.bump());
            } else {
                try!(self.bump());
            }
        }
        let mut conditions = None;
        let mut order_vec = Vec::new();
        // optional where statement
        if self.expect_keyword(&[Keyword::Where]).is_ok() {
            conditions = Some(try!(self.parse_where_part()));
        }
        if self.expect_keyword(&[Keyword::Group]).is_ok() {
            try!(self.bump());
            try!(self.expect_keyword(&[Keyword::By]));
            try!(self.bump());
            return Err(ParseError::DebugError("GroupBy part needs implementation!".to_string()));
        }
        if self.expect_keyword(&[Keyword::Order]).is_ok() {
            try!(self.bump());
            try!(self.expect_keyword(&[Keyword::By]));
            let mut done = false;
            while !done
            {
                try!(self.bump());
                let mut o_alias = None;
                if self.check_next_token(&[Token::Dot]) {
                    o_alias = Some(try!(self.expect_word(false)));
                    try!(self.bump());
                    try!(self.bump());
                };
                let o_col = try!(self.expect_word(true));
                let mut o_order = Some(Order::Asc);
                if self.check_next_keyword(&[Keyword::Asc]) {
                    try!(self.bump());
                } else if self.check_next_keyword(&[Keyword::Desc]) {
                    o_order = Some(Order::Desc);
                    try!(self.bump());
                }
                order_vec.push(Sort {
                    alias: o_alias,
                    col: o_col,
                    order: o_order,
                });
                if !self.check_next_token(&[Token::Comma]) {
                    done = true;
                } else {
                    try!(self.bump());
                }
            }
            if self.check_next_keyword(&[Keyword::Limit]) {
                try!(self.bump());
            }
        }
        let mut limit = None;
        if self.expect_keyword(&[Keyword::Limit]).is_ok() {
            try!(self.bump());
            let tmp = match try!(self.expect_number()) {
                Lit::Int(i) => i ,
                _ => return Err(ParseError::LimitError) ,
            };
            if self.check_next_token(&[Token::Comma]) {
                try!(self.bump());
                try!(self.bump());
                let count = match try!(self.expect_number()) {
                    Lit::Int(i) => i ,
                     _ => return Err(ParseError::LimitError) ,
                };
                limit = Some(Limit { count: Some(count), offset: Some(tmp) } ) ;
            } else {
                limit = Some(Limit { count: Some(tmp) , offset: None} );
            };
        }
        Ok(SelectStmt {
            target: targetvec,
            tid: tidvec,
            alias: aliasmap,
            cond: conditions,
            spec_op: None,
            order : order_vec,
            limit: limit,
        })
    }


// ============================================================================
// Utility Functions
// ============================================================================

    // sets next position for the lexer
    fn bump(&mut self) -> Result<(),ParseError> {
        swap(&mut self.last, &mut self.curr);  //  last = curr
        swap(&mut self.curr, &mut self.peek);  //  curr = peek
        self.peek = try!(self.lexiter.next_real());
        Ok(())
    }
    // checks, if query is ended correctly. if yes -> returns query as ast
    fn return_query_ast(&mut self, query: Query) -> Result<Query, ParseError> {
        try!(self.bump());
        if self.curr.is_none() {
            Ok(query)
        } else {
            Err(ParseError::InvalidEoq)
        }
    }
    // parses the where part into Conditions type
    fn parse_where_part(&mut self) -> Result<Conditions, ParseError> {
        let mut cond;
        if self.check_next_token(&[Token::ParenOp]) {
            try!(self.bump());
            cond = try!(self.parse_where_part());
            try!(self.expect_token(&[Token::ParenCl]).map_err(|e| match e {
                ParseError::WrongToken(span) => ParseError::MissingParenthesis(span),
                _ => e,
            }));
            if self.check_next_keyword(&[Keyword::Or,Keyword::And]) {
                try!(self.bump());
                if self.expect_keyword(&[Keyword::Or]).is_ok() {
                    cond = Conditions::Or(
                        Box::new(cond),Box::new(try!(self.parse_where_part())));
                } else if self.expect_keyword(&[Keyword::And]).is_ok(){
                    cond = Conditions::And(
                        Box::new(cond),Box::new(try!(self.parse_where_part())));
                };
            }
        } else {
            cond = Conditions::Leaf(try!(self.parse_condition()));
            try!(self.bump());
            while self.expect_keyword(&[Keyword::And, Keyword::Or]).is_ok() {
                if self.expect_keyword(&[Keyword::Or]).is_ok() {
                    cond = Conditions::Or(Box::new(cond),Box::new(try!(self.parse_where_part())));
                } else {
                    if self.check_next_token(&[Token::ParenOp]) {
                        cond = Conditions::And(
                            Box::new(cond),
                            Box::new(try!(self.parse_where_part())));
                    } else {
                        cond = Conditions::And(Box::new(cond),
                            Box::new(Conditions::Leaf(try!(self.parse_condition()))));
                        try!(self.bump());
                    };
                };
            };
        }
        Ok(cond)
    }

    fn check_next_token(&self, checktoken: &[Token]) -> bool {
        match self.peek {
            Some(ref token) => { checktoken.contains(&token.tok)},
             _ => false,
        }
    }

    fn check_next_keyword(&self, checkkeyword: &[Keyword]) -> bool {
        let tokenspan = match self.peek {
            Some(ref s) => s.clone(),
             _ => return false,
        };
        let possiblekeyword = match tokenspan.tok {
            Token::Word(ref s) => s,
            _ => return false,
        };
        match keyword_from_string(possiblekeyword) {
            Some(found_keyword) => checkkeyword.contains(&found_keyword),
            None => false
        }
    }
    // aprses a single condition
    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        try!(self.bump());
        let mut alias = None;
        if self.check_next_token(&[Token::Dot]) {
            alias = Some(try!(self.expect_word(false)));
            try!(self.bump());
            try!(self.bump());
        };
        let columnname = try!(self.expect_word(true));
        try!(self.bump());
        let operation = match try!(self.expect_token(&[Token::Equ, Token::GThan,
        Token::SThan, Token::GEThan,
        Token::NEqu, Token::SEThan])) {
            Token::Equ => CompType::Equ,
            Token::GThan => CompType::GThan,
            Token::SThan => CompType::SThan,
            Token::SEThan => CompType::SEThan,
            Token::GEThan => CompType::GEThan,
            Token::NEqu => CompType::NEqu,
            _ => return Err(ParseError::UnknownError),

        };
        try!(self.bump());
        let mut rhsalias = None;
        let rhs = match self.expect_word(false) {
            Ok(s) => {
                if self.check_next_token(&[Token::Dot]) {
                    rhsalias = Some(s);
                    try!(self.bump());
                    try!(self.bump());
                }
                CondType::Word(try!(self.expect_word(true)))
            },
            _ => CondType::Literal(try!(self.expect_literal())),
        };
        Ok(Condition {
            aliascol: alias,
            col: columnname,
            op: operation,
            aliasrhs: rhsalias,
            rhs: rhs,
        })
    }
    // Utility function to parse metadata of columns
    fn expect_column_info(&mut self) -> Result<ColumnInfo, ParseError> {
        let column_id = try!(self.expect_word(true));
        try!(self.bump());
        let dtype = try!(self.expect_datatype());
        let mut colprimary = false;
        let mut auto_increment = false;
        let mut not_null = false;
        let mut comment = None;

        while self.peek.is_some() && !self.check_next_token(&[Token::ParenCl, Token::Comma]) {
            if self.check_next_keyword(&[Keyword::Primary]) {
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Primary]));
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Key]));
                colprimary = true;
            } else if self.check_next_keyword(&[Keyword::AutoIncrement]) {
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::AutoIncrement]));
                auto_increment = true;
            } else if self.check_next_keyword(&[Keyword::Not]) {
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Not]));
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Null]));
                not_null = true;
            } else if self.check_next_keyword(&[Keyword::Comment]) {
                try!(self.bump());
                try!(self.expect_keyword(&[Keyword::Comment]));
                try!(self.bump());
                let tmp = try!(self.expect_literal());
                comment = Some(match tmp {
                    Lit::String(s) => s,
                    _ => return Err(ParseError::CommentIsNoString),
                })
            } else {
                break;
            }
        }

        Ok(ColumnInfo {
            cid: column_id,
            datatype: dtype,
            primary: colprimary,
            auto_increment: auto_increment,
            not_null: not_null,
            comment: comment,
        })
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
            span_lo = token.span.lo;
            span_hi = token.span.hi;
            // checks whether token is a word
            let word = match token.tok {
                Token::Word(ref s) => s,
                _ => return Err(ParseError::NotADatatype(Span { lo: span_lo , hi: span_hi } ))
            };
            tmp_datatype = word.to_lowercase();
        }
        // checks if token is a correct Datatype
        found_datatype = match &tmp_datatype[..] {
            "int" => SqlType::Int,
            "bool" => SqlType::Bool,
            "boolean" => SqlType::Bool,
            // checks if char is written in correct sql syntax
            "char" => {
                try!(self.bump());
                try!(self.expect_token(&[Token::ParenOp]));
                try!(self.bump());
                let length_lit = try!(self.expect_number());
                try!(self.bump());
                try!(self.expect_token(&[Token::ParenCl]));

                let length = match length_lit {
                    Lit::Int(i) => {
                        if 0 <= i && i <= ( u8::max_value() as i64)  {
                            i as u8
                        }else {
                            return Err(ParseError::DatatypeMissmatch(
                                Span { lo: span_lo , hi: span_hi }
                            ))
                        }
                    },
                    _ => return Err(ParseError::DatatypeMissmatch(
                                Span { lo: span_lo , hi: span_hi }
                                ))
                };
                SqlType::Char(length)
            },
            _ => return Err(ParseError::NotADatatype(
             Span { lo: span_lo , hi: span_hi }
             )),
        };
        Ok((found_datatype))
    }
    // checks if the current token is a word
    fn expect_word(&self,allowkeyword: bool) -> Result<String, ParseError> {
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
            span_lo = token.span.lo;
            span_hi = token.span.hi;
            // checks whether token is a word
            found_word = match token.tok {
                Token::Word(ref s) => s,
                _ => return Err(ParseError::NotAWord(
                 Span { lo: span_lo , hi: span_hi }
                 ))
            };
        }
        if keyword_from_string(found_word).is_some() && !allowkeyword {
            Err(ParseError::ReservedKeyword(Span { lo: span_lo , hi: span_hi }))
        } else {
            Ok(found_word.to_string())
        }
    }

    // checks if the current token is a word
    fn expect_literal(&self) -> Result<Lit, ParseError> {
        let mut found_lit;
        let mut span_lo;
        let mut span_hi;
        {
            // checks if token non or some
            let token = match self.curr {
                None => return Err(ParseError::UnexpectedEoq),
                // in case of som: return reference to token
                Some(ref token) => token,
            };
            span_lo = token.span.lo;
            span_hi = token.span.hi;
            // checks whether token is a word
            found_lit = match token.tok {
                Token::Word(ref s) => {
                    if s.to_lowercase() == "true" {

                        Lit::Bool(1)
                    } else if s.to_lowercase() == "false" {
                        Lit::Bool(0)
                    } else {
                        return  Err(ParseError::NotALiteral(
                         Span { lo: span_lo , hi: span_hi } ))
                    }
                }
                Token::Literal(ref s) => s.clone(),
                _ => return Err(ParseError::NotALiteral(
                 Span { lo: span_lo , hi: span_hi }
                 ))
            };
        }
        Ok(found_lit)
    }
    // checks if the current token is a number
    fn expect_number(&self) -> Result<Lit, ParseError> {
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
            span_lo = token.span.lo;
            span_hi = token.span.hi;
            // checks whether token is a valid number
            found_num = match token.tok {
                Token::Literal(Lit::Int(s)) => Lit::Int(s),
                Token::Literal(Lit::Float(s)) => Lit::Float(s),
                _ => return Err(ParseError::NotANumber(Span { lo: span_lo , hi: span_hi } ))
            };
        }
        Ok(found_num)
    }
    // checks if current token is an expected token
    fn expect_token(&self,expected_tokens: &[Token])
    -> Result<Token, ParseError>
    {
            // checks if current is none or some
            let token = match self.curr {
                None => return Err(ParseError::UnexpectedEoq),
                // in case of some: return reference to token
                Some(ref token) => token,
            };
            if expected_tokens.contains(&(token.tok)) {
                Ok(token.tok.clone())
            } else {
                Err(ParseError::WrongToken(Span { lo: token.span.lo, hi: token.span.hi } ))
            }
    }
    // matches current token against any keyword and checks if it is one of
    // the expected keywords
    fn expect_keyword(&self,expected_keywords: &[Keyword])
    -> Result<Keyword, ParseError>
    {
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
            span_lo = token.span.lo;
            span_hi = token.span.hi;
            // checks whether token is a word
            let word = match token.tok {
                Token::Word(ref s) => s,
                _ => return Err(ParseError::NotAKeyword(Span { lo: span_lo , hi: span_hi } ))
            };
            // checks if word is a keyword
            found_keyword = match keyword_from_string(&word){
                Some(keyword) => keyword,
                None => return Err(ParseError::NotAKeyword(Span { lo: span_lo , hi: span_hi } )),
            };
        }
        // checks if keyword is expected keyword
        if expected_keywords.contains(&found_keyword) {
            Ok(found_keyword)
        } else {
            Err(ParseError::WrongKeyword(Span { lo: span_lo , hi: span_hi } ))
        }
    }
}

fn keyword_from_string(string: &str) -> Option<Keyword> {
    let tmp = string.to_lowercase();
    match &tmp[..]{
                "create" => Some(Keyword::Create),
                "drop" => Some(Keyword::Drop),
                "table" => Some(Keyword::Table),
                "view" => Some(Keyword::View),
                "alter" => Some(Keyword::Alter),
                "update" => Some(Keyword::Update),
                "select" => Some(Keyword::Select),
                "insert" => Some(Keyword::Insert),
                "delete" => Some(Keyword::Delete),
                "modify" => Some(Keyword::Modify),
                "add" => Some(Keyword::Add),
                "column" => Some(Keyword::Column),
                "database" => Some(Keyword::Database),
                "into" => Some(Keyword::Into),
                "use" => Some(Keyword::Use),
                "values" => Some(Keyword::Values),
                "from" => Some(Keyword::From),
                "where" => Some(Keyword::Where),
                "describe" => Some(Keyword::Describe),
                "and" => Some(Keyword::And),
                "or" => Some(Keyword::Or),
                "set" => Some(Keyword::Set),
                "as" => Some(Keyword::As),
                "primary" => Some(Keyword::Primary),
                "key" => Some(Keyword::Key),
                "group" => Some(Keyword::Group),
                "by" => Some(Keyword::By),
                "having" => Some(Keyword::Having),
                "order" => Some(Keyword::Order),
                "desc" => Some(Keyword::Desc),
                "asc" => Some(Keyword::Asc),
                "limit" => Some(Keyword::Limit),
                "replace" => Some(Keyword::Replace),
                "auto_increment" => Some(Keyword::AutoIncrement),
                "not" => Some(Keyword::Not),
                "null" => Some(Keyword::Null),
                "comment" => Some(Keyword::Comment),
                _ => None,
            }
}


// ===========================================================================
// Enums
// ===========================================================================

#[derive(PartialEq)]
// Keywords that can occour in SQL syntax
pub enum Keyword {
    // 1st level keywords
    // data definition keywords
    Create,
    Drop,
    Alter,
    Use,
    Describe,
    // data manipulation keywords
    Select,
    Update,
    Insert,
    Delete,
    Set,
    // 2nd level keywords
    Table,
    Database,
    View,
    Column,
    // 3rd level keywords
    From,
    Where,
    Group,
    Order,
    Having,
    Limit,
    Modify,
    Add,
    Into,
    Values,
    And,
    Or,
    As,
    By,
    Asc,
    Desc,
    Primary,
    Key,
    Replace,
    AutoIncrement,
    Not,
    Null,
    Comment,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    LexError(lex::LexError),
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
    NotALiteral(Span),
    ColumnCountMissmatch,
    MissingParenthesis(Span),
    LimitError,
    ReservedKeyword(Span),
    CommentIsNoString,
    //Used for debugging
    DebugError(String)
// TODO: introduce good errors and think more about it
}

impl From<lex::LexError> for ParseError {
    fn from(error: lex::LexError) -> ParseError {
        ParseError::LexError(error)
    }
}
