///  Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;



fn main() {

    let mut p = parse::Parser::create("drop table bla");
    println!("{:?}",p.parse());


}
