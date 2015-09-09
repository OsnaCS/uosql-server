///  Program for testing and playing with the parser
///

extern crate server;
use server::parse;



fn main() {

    let mut p = parse::Parser::create("select * from foo limit 30,3");
    println!("{:?}",p.parse());

}
