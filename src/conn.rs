//! Contains the entry point code for handling an incoming connection.
//!

use std::net::TcpStream;

pub fn handle(stream: TcpStream) {
    let addr = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or("???".into());
    info!("Handling connection from {}", addr);
}
