//! Contains the entry point code for handling an incoming connection.
//!
use std::net::TcpStream;
use net;
use auth;
use parse::parser;
use super::query;
use net::types::*;

pub fn handle(mut stream: TcpStream) {
    // Logging about the new connection
    let addr = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or("???".into());
    info!("Handling connection from {}", addr);

    // Perform handshake, check user login --> done
    let res = net::do_handshake(&mut stream);
    let _ = match res {
        Ok((user, pw)) => {
            info!("Connection established. Handshake sent");
            match auth::find_user(&user, &pw) {
                Ok(_) => {
                    match net::send_info_package(&mut stream,
                        PkgType::AccGranted)
                    {
                        Ok(_) => {},
                        Err(_) => return
                    }
                },
                Err(_) => {
                    let _ =
                        net::send_info_package(&mut stream, PkgType::AccDenied);
                    // Loops for user-convienience?
                    return
                }
            }
        },
        _ => {
            let _ = net::send_info_package(&mut stream, PkgType::AccDenied);
            return
        }
    };

    // Read commands from the client (with help of `net`) --> done
    loop {
        //get the command from the stream
        let command_res = net::read_commands(&mut stream);

        // TODO: Dispatch commands (handle easy ones directly, forward others)
        match command_res {
            Ok(cmd) =>
            match cmd {
                // exit the session and shutdown the connection
                Command::Quit => {
                    match net::send_info_package(&mut stream, PkgType::Ok) {
                        Ok(_) => {
                            debug!("Client disconnected properly.");
                            return
                        },
                        Err(_) =>
                            warn!("Failed to send packet. Connection close.")
                    }
                },
                // send OK-Package, unused value can be checked to try again and
                // eventually close to connection as timeout issue
                Command::Ping => {
                    match net::send_info_package(&mut stream, PkgType::Ok) {
                        Ok(_) => { },
                        Err(_) => warn!("Failed to send packet.")
                    }
                },
                // send the query string for parsing
                // TODO: If query -> Call parser to obtain AST
                // TODO: If query -> Pass AST to query executer
                // TODO: Send results
                Command::Query(q) => {

                    debug!("Query received, dispatch query to parser.");

                    let mut p = parser::Parser::create(&q);
                    let ast = p.parse();

                    match ast {
                        Ok(tree) => {
                            println!("{:?}", tree);
                            query::execute_from_ast(tree, & mut auth::User { _name: "DummyUser".into(), _currentDatabase: None} );
                        },
                        Err(error) => println!("{:?}", error),
                    }

                    // TODO: Definition of response missing
                    match net::send_info_package(&mut stream, PkgType::Ok) {
                        Ok(_) => { },
                        Err(_) => warn!("Failed to send packet.")
                    }
                    continue
                }
            },
            Err(_) => continue // TODO: error handling
        }
    }
}
