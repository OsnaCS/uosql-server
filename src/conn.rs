//! Contains the entry point code for handling an incoming connection.
//!
use std::net::TcpStream;
use net;
use net::Command;
use auth;

pub fn handle(mut stream: TcpStream) {
    // Logging about the new connection
    let addr = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or("???".into());
    info!("Handling connection from {}", addr);

    // Perform handshake, check user login --> done
    let res = net::do_handshake(&mut stream);
    let auth_res = match res {
        Ok((user, pw)) => auth::find_user(&user, &pw),
        _ => Err(auth::AuthError::UserNotFound)
    };

    // TODO: Read commands from the client (with help of `net`)
    loop {
        //get the command from the stream
        let command_res = net::read_commands(&mut stream);
        println!("{:?}",command_res);
        // TODO !
        match command_res {
            Ok(Command::Quit) => return,
            _ => continue
        }
        // TODO: Dispatch commands (handle easy ones directly, forward others)

        // TODO: If query -> Call parser to obtain AST
        // TODO: If query -> Pass AST to query executer

        // TODO: Send results
    }
}
