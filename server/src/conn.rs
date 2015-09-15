//! Contains the entry point code for handling an incoming connection.
//!
use std::net::TcpStream;
use net;
use auth;
use parse;
use super::query;
use net::types::*;
use storage::{Rows};
use storage::types::{SqlType, Column};
use std::error::Error;
use std::io::{Cursor};

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
                        Err(e) => { error!("{}", e.description()); return }
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

                    let ast = parse::parse(&q);

                    match ast {
                        Ok(tree) => {
                            debug!("{:?}", tree);

                            let r = query::execute_from_ast(tree, & mut auth::User {
                                _name: "DummyUser".into(),
                                _currentDatabase: None} ).
                                unwrap_or(
                                    Rows::new(Cursor::new(Vec::new()), &vec![
                                        Column::new("error occurred", SqlType::Int, false,
                                        "error mind the error, not an error again, I hate errors",
                                        false)])

                                );

                            // Send response package
                            match net::send_response_package(&mut stream, r) {
                                Ok(_) => { },
                                Err(_) => warn!("Failed to send packet.")
                            }
                        },

                        Err(error) => {
                            error!("{:?}", error);
                            match net::send_error_package(&mut stream,
                                net::Error::UnEoq(error).into())
                            {
                                Ok(_) => {},
                                Err(_) => warn!("Failed to send error.")
                            }
                        }
                    }
                    continue
                }
            },
            Err(_) => continue // TODO: error handling
        }
    }
}
