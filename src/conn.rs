//! Contains the entry point code for handling an incoming connection.
//!
use std::net::TcpStream;
use net;

pub fn handle(mut stream: TcpStream) {
    // Logging about the new connection
    let addr = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or("???".into());
    info!("Handling connection from {}", addr);

    // TODO: Perform handshake --> done
    net::do_handshake(&mut stream);

    // TODO: check user login

    // TODO: Read commands from the client (with help of `net`)

    // TODO: Dispatch commands (handle easy ones directly, forward others)

    // TODO: If query -> Call parser to obtain AST
    // TODO: If query -> Pass AST to query executer

    // TODO: Send results
}
