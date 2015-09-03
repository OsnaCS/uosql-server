///  Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;



fn main() {


    let mut p = parse::Parser::create("alter table test modify stuff bla");
    println!("{:?}",p.parse());

}
