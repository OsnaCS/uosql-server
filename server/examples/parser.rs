///  Program for testing and playing with the parser
///

extern crate server;
use server::parse;



fn main() {

    let mut p = parse::Parser::create("select x,y,z from a,b,c,d,e,f,g where x=y and z=54");
    println!("{:?}",p.parse());

}
