///  Program for testing and playing with the parser
///

extern crate server;
use server::parse;



fn main() {

    let mut p = parse::Parser::create("delete from Stephanie where (x = 8) and ((Test = 2) or x = 3) and (ljrhglejrgl='asdasdasd')");
    println!("{:?}",p.parse());

}
