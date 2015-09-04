//! Contains the entry point code for handling an incoming connection.
//!
use std::net::TcpStream;
use net;
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

    // Read commands from the client (with help of `net`) --> done
    loop {
        //get the command from the stream
        let command_res = net::read_commands(&mut stream);
        
        // TODO: Dispatch commands (handle easy ones directly, forward others)
        match command_res {
            Ok(cmd) => 
            match cmd {
                //exit the session and shutdown the connection
                net::Command::Quit => { net::send_ok_packet(&mut stream); return }, 
                // send OK-Package, unused value can be checked to try again and 
                // eventually close to connection as timeout issue
                net::Command::Ping => { net::send_ok_packet(&mut stream); } , 
                // send the query string for parsing
                // TODO: If query -> Call parser to obtain AST
                // TODO: If query -> Pass AST to query executer
                // TODO: Send results
                net::Command::Query(query) => continue
            },
            Err(e) => continue//error handling
        }
    } 
}
