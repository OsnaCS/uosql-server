//! Simple client program
//! Establishes connection to server and sends login information
#[macro_use]
extern crate log;
extern crate uosql;
extern crate bincode;
extern crate byteorder;

use std::io::{self, stdout, Write};
use uosql::logger;
use uosql::Error;
use uosql::Connection;

fn main() {

    logger::with_loglevel(log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();

    // IP, Port, Username, Passwort einlesen

    let mut conn = match Connection::connect("127.0.0.1".into(), 4242,
        "hallo".into(), "bla".into())
    {
        Ok(conn) => conn,
        Err(e) => {
            match e {
                Error::AddrParse(_) => {
                    error!("Could not connect to specified server.");
                    return
                },
                Error::Io(_) => {
                    error!("Connection failure. Try again later.");
                    return
                },
                Error::Decode(_) => {
                    error!("Could not read data from server.");
                    return
                },
                Error::Encode(_) => {
                    error!("Could not send data to server.");
                    return
                },
                Error::UnexpectedPkg(e) => {
                    error!("{}", e.to_string());
                    return
                },
                Error::Auth(e) => {
                    info!("{}", e.to_string());
                    return
                }
            }
        }
    };

    println!("Connected (version: {}) to {}:{}\n{}\n",
        conn.get_version(), conn.get_ip(), conn.get_port(), conn.get_message());

    // read commands
    loop {
        print!("> ");
        let e = stdout().flush();
        match e {
            Ok(_) => {},
            Err(_) => info!("")
        }
        let input = read_line();

        // send code for command-package
        let cs = process_input(&input, &mut conn);
        match cs {
            false => return, // end client
            true => continue, // next iteration
        }
    }
}

/// Read from command line and return trimmed string
/// If an error occurs reading from stdin loop until a valid String was read
fn read_line() -> String {
    let mut input = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(_) => { return input.trim().into() },
            _ => { }
        }
    }
}

/// Process commandline-input from user
fn process_input(input: &str, conn: &mut Connection) -> bool {
    let input_low = input.to_lowercase();

    match &*input_low {
        ":quit" => {
            match conn.quit() {
                Ok(_) => return false,
                Err(_) => {
                    error!("Sending quit-message failed");
                    return true
                }
            }
        },
        ":ping" => {
            match conn.ping() {
                Ok(_) => {
                    println!("Server still reachable.");
                    return true
                },
                Err(_) => {
                    error!("Sending ping-message failed");
                    return true
                }
            }
        },
        ":exit" => {
            // TODO: other functionality to exit than quit
            match conn.quit() {
                Ok(_) => {
                    println!("Bye bye.");
                    return false
                },
                Err(_) => {
                    error!("Sending quit-message failed");
                    return false
                }
            }
        },
        ":help" => {
            let help = include_str!("readme.txt");
            println!("{}", help);
        },
        _ => {
            // Size
            match conn.execute(input.into()) {
                Ok(_) => { println!("Query sent. Waiting for response.");},
                Err(e) => {
                    match e {
                        Error::Io(_) => {
                            error!("Connection failure. Try again later.");
                            return true
                        },
                        Error::Decode(_) => {
                            error!("Could not read data from server.");
                            return true
                        }
                        Error::Encode(_) => {
                            error!("Could not send data to server.");
                            return true
                        }
                        Error::UnexpectedPkg(e) => {
                            error!("{}", e.to_string());
                            return true
                        },
                        _ => {
                            error!("Unexpected behaviour during execute()");
                            return false
                        }
                    }
                }
            }
        }
    }
    true
}
