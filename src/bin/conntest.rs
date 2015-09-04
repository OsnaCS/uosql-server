extern crate uosql;
extern crate byteorder;
extern crate rustc_serialize;
extern crate bincode;

use uosql::net;
//use bincode::rustc_serialize::{encode_into};
//use bincode::SizeLimit;

/// test of some functions and structs by net
fn main() {
    let mut vec = Vec::new();   // stream to write into
   

    let err = net::NetworkErrors::UnexpectedPkg("unexpected".into());

    //test if the message is sent
    let res = net::send_error_packet(&mut vec, err.into());
    println!("{:?}", vec );
}
