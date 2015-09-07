///  Program for testing and playing with the parser
///

extern crate server;
use server::parse;



fn main() {

    let mut p = parse::Parser::create("delete from Stephanie where a='c0' and x = 'c1' or y = 'c2' and z = 'c3'");
    println!("{:?}",p.parse());

}
