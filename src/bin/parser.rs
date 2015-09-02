/// Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;
use std::iter::Iterator;
use uosql::parse::ast::{Query, DefStmt, CreateStmt, DropStmt, CreateTableStmt};
use uosql::parse::token::TokenSpan;
use uosql::parse::lex::Lexer;
use std::mem::swap;
use uosql::parse::token::Token;

pub enum Keyword {
    Create,
    Drop,
    Alter,
    Select

}





struct Parser<'a>{
    lexiter: Lexer<'a>,
    curr: Option<TokenSpan>,
    peek: Option<TokenSpan>
}





impl<'a> Parser<'a>{

    fn create(query: &'a str) -> Parser<'a>{
        let l = parse::lex::Lexer::from_query(query);
        let mut p= Parser{lexiter:l,curr:None,peek:None};
        p.lexer_next();
        p.lexer_next();
        p
    }


    fn lexer_next(&mut self){
        swap(&mut self.curr, &mut self.peek);  // curr = peek
        self.peek = self.lexiter.next();

    }



    ///Parses the given query into an AST
    fn parse(&mut self) -> Result<parse::ast::Query, ParseError>{

        let querytype = self.expect_keyword(&[Keyword::Create,Keyword::Drop, Keyword::Alter, Keyword::Select]);

        match querytype {
            Err(ParseError::EofError) => Err(ParseError::EmptyQueryError),
            Err(error) => Err(error),
            Ok(Keyword::Create) => Ok(Query::DefStmt(DefStmt::Create(self.parse_create_stmt()))),
            Ok(Keyword::Drop) => Ok(Query::DefStmt(DefStmt::Drop(self.parse_drop_stmt()))),
            _=>Err(ParseError::HugeError)
        }
    }



    fn parse_create_stmt(&mut self)  -> CreateStmt {
        CreateStmt::Table(CreateTableStmt {tid:"bglkjnxfhoksdtop7z".to_string(), cols: None})

        //let type = self.expect_keyword(Table);
        //match table/ view create...




    }

    fn parse_drop_stmt(&mut self)  -> DropStmt {
        DropStmt::Drop
        //let type = self.expect_keyword(Table);

    }

    fn expect_keyword(&mut self,expected_keywords: &[Keyword]) -> Result<Keyword, ParseError> {

        if(self.curr.is_none()){
            return Err(ParseError::EofError);
        }



        Ok(Keyword::Create)
    }
}







#[derive(Debug)]
enum ParseError {
    HugeError,
    EmptyQueryError,
    EofError
//TODO: introduce good errors and think more about it
}




fn main() {

	/*let l = parse::lex::Lexer::from_query("hi, Lisa! Wie geht es dir?");
	for i in l {
		println!("{:?}", i.tok);
	}*/



    let mut p = Parser::create("Create Table X");
    println!("{:?}",p.parse());

}
