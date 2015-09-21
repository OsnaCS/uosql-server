///  Program for testing and playing with the parser
///

extern crate server;
use server::parse;



fn main() {

    let mut p = parse::Parser::create("insert into foo values ('⊂(▀¯▀⊂)', 420, 'lel')");
    println!("{:?}",p.parse());

}
