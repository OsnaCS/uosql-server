///  Program for testing and playing with the parser
///

extern crate server;
use server::parse;



fn main() {

    let mut p = parse::Parser::create("select s.* as AllesStudenten, h.* as AllesHoeren from Studenten s, hoeren h where s.matrNr = h.matrNr Limit 0,100");
    println!("{:?}",p.parse());

}
