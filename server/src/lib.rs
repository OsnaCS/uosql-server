extern crate bincode;
extern crate byteorder;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate term_painter as term;

pub mod auth;
pub mod conn;
pub mod logger;
pub mod net;
pub mod parse;
pub mod query;
pub mod storage;

use std::net::{Ipv4Addr, SocketAddrV4};

/// A struct for managing configurations
#[derive(Debug)]
pub struct Config {
    pub address: Ipv4Addr,
    pub port: u16,
    pub dir: String
}

/// Listens for incoming TCP streams
pub fn listen(config: Config) {
    use std::net::TcpListener;
    use std::thread;

    // Converting configurations to a valid socket address
    let sock_addr = SocketAddrV4::new(config.address, config.port);
    let listener = TcpListener::bind(sock_addr).unwrap();

    // Accept connections and process them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Connection succeeded: Spawn thread and handle
                thread::spawn(move|| {
                    conn::handle(stream)
                });
            },
            Err(e) => {
                // Something went wrong...
                warn!("Failed to accept incoming connection: {:?}", e);
            },
        }
    }
}
