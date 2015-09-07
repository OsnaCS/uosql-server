//! Simple client program
//! Establishes connection to server and sends login information
#[macro_use]
extern crate log;
extern crate uosql;
extern crate bincode;
extern crate byteorder;

use std::io::{self, stdout, Write, Read};
use std::net::TcpStream;
use uosql::logger;
use uosql::net::{PkgType, Greeting, Login, Command, Error};
use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};

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
        let cs = process_input(&mut s, &input);
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

fn send_cmd<W: Write>(mut s: &mut W, cmd: Command, size: u64) -> Result<(), Error> {
    try!(encode_into(&PkgType::Command, s, SizeLimit::Bounded(1)));
    try!(encode_into(&cmd, &mut s, SizeLimit::Bounded(size)));
    Ok(())
}

/// Process commandline-input from user
fn process_input<R: Read + Write>(mut s: &mut R, input: &str) -> bool {
    let input_low = input.to_lowercase();

    match &*input_low {
        ":quit" => {
            match send_cmd(s, Command::Quit, 1) {
                Ok(_) => {
                    match receive(&mut s, PkgType::Ok) {
                        Ok(_) => { info!("Connection closed"); return true },
                        Err(_) => {
                            warn!("Failed to receive close-confirmation");
                            return true
                        }
                    }
                },
                Err(_) => {
                    error!("Sending quit-message failed");
                    return false
                }
            }
        },
        ":ping" => {
            match send_cmd(s, Command::Ping, 1) {
                Ok(_) => {},
                Err(_) => {
                    error!("Sending ping-message failed");
                    return false
                }
            };
            match receive(&mut s, PkgType::Ok) {
                Ok(_) => println!("Server still reachable."),
                Err(_) => error!("Error reading ping-package")
            }
        },
        ":exit" => {
            // TODO: other functionality to exit than quit
            match send_cmd(s, Command::Quit, 1) {
                Ok(_) => {
                    match receive(&mut s, PkgType::Ok) {
                        Ok(_) => { info!("Connection closed"); return true },
                        Err(_) => {
                            warn!("Failed to receive close-confirmation");
                            return true
                        }
                    }
                },
                Err(_) => {
                    error!("Sending quit-message failed");
                    return true
                }
            }
        },
        ":help" => {
            let help = include_str!("readme.txt");
            println!("{}", help);
        },
        _ => {
            // Size
            match send_cmd(s, Command::Query(input.into()), 1024) {
                Ok(_) => {},
                Err(_) => {
                    error!("Sending command-package failed. Try again.");
                    return false
                }
            };
            match receive(&mut s, PkgType::Ok) {
                Ok(_) => warn!("decoding response not implemented yet!"),
                Err(_) => error!("Error reading response-package")
            }
        }
    }
    false
}

/// Match received packages to expected packages
fn receive<R: Read>(s: &mut R, cmd: PkgType) -> Result<(), Error> {
    let status: PkgType = try!(decode_from(s, SizeLimit::Bounded(1)));

    if status != cmd {
        return Err(Error::UnexpectedPkg("Received
            unexpected package".into()))
    }
    Ok(())
}

/// Receive greeting from server
fn receive_greeting<R: Read>(mut buf: &mut R) -> bool {
    match receive(&mut buf, PkgType::Greet) {
        Ok(_) => {},
        Err(_) => {
            info!("Communication mismatch. Try again later.");
            return false
        }
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
    println!("Username: ");
    let usern = read_line();

    println!("Password: ");
    let passw = read_line();
    let login = Login {username: usern, password: passw};

    //send Login package to server
    match encode_into(&PkgType::Login, buf, SizeLimit::Bounded(1)) {
        Ok(_) => {},
        Err(_) => {
            info!("Sending package header failed");
            return false
        }
    }

    match encode_into(&login, buf, SizeLimit::Bounded(1024)) {
        Ok(_) => return true,
        Err(_) => {
            info!("Sending login-data failed");
            return false
        }
    }
}
