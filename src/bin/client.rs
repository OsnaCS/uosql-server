//! Simple client program
//! Establishes connection to server and sends login information
#[macro_use]
extern crate log;
extern crate uosql;
extern crate bincode;
extern crate byteorder;

use uosql::logger;
use std::io::{self, stdout, Write, Read};
use std::net::TcpStream;
use uosql::net::{Cnv, Greeting, Login, Command};
use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};
use byteorder::{ReadBytesExt, WriteBytesExt};

const PROTOCOL_VERSION : u8 = 1;

fn main() {
    logger::with_loglevel(log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();
    // connect to server
    let stream = TcpStream::connect("127.0.0.1:4242");
    let mut s = match stream {
        Ok(s) => s,
        Err(_) => { error!("Could not connect to server."); return }
    };
    info!("Connected");

    // receive welcome message from server
    let rg = receive_greeting(&mut s);
    if !rg {
        info!("Connection closed.");
        return;
    }

    // try to send login-data as long as it didn't succeed
    while !send_login(&mut s) {}

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
        let cs = send_cmd(&mut s, &input);
        match cs {
            true => return, // end client
            false => continue, // next iteration
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

/// Send command-package to server.
fn send_cmd<R: Read + Write>(mut s: &mut R, input: &String) -> bool {
    let input_low = input.to_lowercase();
    let status = s.write_u8(Cnv::CommandPkg as u8);
    let _ = match status {
        Ok(_) => {},
        Err(_) => {
            error!("Sending command-header failed");
            return false
        }
    };
    match &*input_low {
        "quit" => {
            let cmd_encode = encode_into(&Command::Quit, &mut s,
                SizeLimit::Bounded(1024));
            match cmd_encode {
                Ok(_) => {
                    let status = s.read_u8();
                    let st = match status {
                        Ok(st) => st,
                        Err(_) => {
                            warn!("Failed to receive close-confirmation");
                            return true
                        }
                    };
                    if st == Cnv::OkPkg as u8 {
                        info!("Connection closed");
                    }
                    return true
                },
                Err(_) => {
                    error!("Sending quit-message failed");
                    return false
                }
            }
        },
        "ping" => {
            let cmd_encode = encode_into(&Command::Ping, &mut s,
                SizeLimit::Bounded(1024));
            let _ = match cmd_encode {
                Ok(_) => {},
                Err(_) => {
                    error!("Sending ping-message failed");
                    return false
                }
            };
            let status = s.read_u8();
            match status {
                Ok(st) => {
                    if st == Cnv::OkPkg as u8 {
                        println!("Server still reachable.");
                    } else {
                        info!("Server is responding but INSANE!");
                        //maybe close connection? -> timeout
                    }
                },
                Err(_) => {
                    error!("Error reading ping-package");
                }
            }
        },
        "exit" => {
            return true
        }
        _ => {
            let cmd_encode = encode_into(&Command::Query(input.to_string()),
                &mut s, SizeLimit::Bounded(1024));
            let _ = match cmd_encode {
                Ok(_) => {},
                Err(_) => {
                    error!("Sending command-package failed. Try again.");
                    return false
                }
            };
            let status = s.read_u8();
            match status {
                Ok(st) => {
                    if st == Cnv::OkPkg as u8 {
                        // decode Response
                    } else {
                        error!("Unexpected return");
                        // try again
                    }
                },
                Err(_) => {
                    error!("Error reading response-package");
                }
            }
        }
    }
    false
}

/// Receive greeting from server
fn receive_greeting<R: Read>(buf: &mut R) -> bool {
    let status = buf.read_u8();
    let st = match status {
        Ok(st) => st,
        Err(_) => return false
    };
    if st != Cnv::GreetPkg as u8 {
        info!("Communication mismatch. Try again later.");
        return false;
    }
    // read greeting
    let greet = decode_from::<R, Greeting>(buf, SizeLimit::Bounded(1024));
    let gr = match greet {
        Ok(gr) => gr,
        _ => {
            info!("Could not decode greet-package");
            return false
        }
    };
    let greeting = Greeting::make_greeting(gr.protocol_version, gr.message);
    if PROTOCOL_VERSION != greeting.protocol_version {
        info!("Cannot communicate with server - different versions");
        return false
    }
    println!("Protocol version: {}\n{}", greeting.protocol_version,
        greeting.message);
    true
}

/// Read login information from command line and send it to the server
fn send_login<W: Write>(buf: &mut W) -> bool {
    let mut login = Login::new();
    println!("Username: ");
    login.username = read_line();

    println!("Password: ");
    login.password = read_line();

    //send Login package to server
    let status = buf.write_u8(Cnv::LoginPkg as u8);
    let _ = match status {
        Ok(_) => {},
        Err(_) => {
            info!("Sending package header failed");
            return false
        }
    };
    let encode = encode_into(&login, buf, SizeLimit::Bounded(1024));
    match encode {
        Ok(_) => return true,
        Err(_) => {
            info!("Sending login-data failed");
            return false
        }
    }
}
