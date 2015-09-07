///  Program for testing and playing with the parser
///

extern crate server;
use server::parse;



fn main() {

    let mut p = parse::Parser::create("insert into random_table () values (int,int, int)");
    println!("{:?}",p.parse());

}
