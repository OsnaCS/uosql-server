//! Simple client program
//! Establishes connection to server and sends login information

extern crate uosql;
extern crate bincode;
extern crate bufstream;
extern crate byteorder;

use std::io::{Write, Read};
use std::io;
use std::io::stdout;
use std::net::TcpStream;
use bincode::rustc_serialize::{decode_from, encode_into};
use uosql::net::Cnv;
use uosql::net::Greeting;
use uosql::net::Login;
use uosql::net::Command;
use byteorder::{ReadBytesExt, WriteBytesExt};
use bincode::SizeLimit;

const PROTOCOL_VERSION : u8 = 1;

fn main() {
    // connect to server
    let stream = TcpStream::connect("127.0.0.1:4242");
    match stream {
        Ok(mut s) => {
            println!("Connected");
            let rg = receive_greeting(&mut s);
            if rg == 1 {
                // close connection
                println!("Connection closed.");
                return;
            }

            // try to send login-data as long as it didn't succeed
            let mut sl = 1;
            while sl != 0 {
                sl = send_login(&mut s);
            }

            // read commands
            loop {
                print!("> ");
                let e = stdout().flush();
                match e {
                    Ok(_) => {},
                    Err(_) => println!("")
                }
                let input = cmd_read();

                // send code for command-package
                let cs = cmd_send(&mut s, &input);
                match cs {
                    0 => return, // end client
                    _ => continue, // next iteration

                }
            }
        } ,
        Err(e) => panic!("Could not connect to server: {}", e)
    }
}

/// Read from command line and return trimmed string
fn cmd_read() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_string(),
        _ => "".to_string() //maybe throw PANIC!
    }
}

// send command-package
fn cmd_send<R: Read + Write>(mut s: &mut R, input: &String) -> u8 {
    let input_low = input.to_lowercase();
    let status = s.write_u8(Cnv::CommandPkg as u8);
    match status {
        Ok(_) => {
            match &*input_low {
                "quit" => {
                    let cmd_encode = encode_into(&Command::Quit, &mut s,
                        SizeLimit::Bounded(1024));
                    match cmd_encode {
                        Ok(_) => return 0,
                        Err(_) => {
                            println!("Sending quit-message failed");
                            return 1
                        }
                    }
                },
                "ping" => {
                    let cmd_encode = encode_into(&Command::Ping, &mut s,
                        SizeLimit::Bounded(1024));
                    match cmd_encode {
                        Ok(_) => {
                            let status = s.read_u8();
                            match status {
                                Ok(st) => {
                                    if st == Cnv::OkPkg as u8 {
                                        println!("Server still reachable.");
                                    } else {
                                        println!("Server is responding but INSANE!");
                                        //maybe close connection? -> timeout
                                    }
                                },
                                Err(_) => {
                                    println!("Error reading ping-package");
                                }
                            }
                        }
                        Err(_) => {
                            println!("Sending ping-message failed");
                        }
                    }
                },
                _ => {
                    let cmd_encode = encode_into(&input, &mut s,
                        SizeLimit::Bounded(1024));
                    match cmd_encode {
                        Ok(_) => {
                            let status = s.read_u8();
                            match status {
                                Ok(st) => {
                                    if st == Cnv::ResponsePkg as u8 {
                                        // decode Response
                                    } else {
                                        println!("Unexpected return");
                                        // try again
                                    }
                                },
                                Err(_) => {
                                    println!("Error reading response-package");
                                }
                            }
                        }
                        Err(_) => {
                            println!("Sending command-package failed. Try again.");
                        }
                    }
                }
            }
        }
        Err(_) => {
            println!("Sending command-header failed");
        }
    }
    1
}

/// Receive greeting from server
fn receive_greeting<R: Read>(buf: &mut R) -> u8 {
    let status = buf.read_u8();
    match status {
        Ok(st) => {
            if st != Cnv::GreetPkg as u8 {
                println!("Communication mismatch. Try again later.");
                return 1;
            } else {
                // read greeting
                let greeting: Greeting;
                let greet = decode_from::<R, Greeting>(buf,
                    SizeLimit::Bounded(1024));
                match greet {
                    Ok(gr) => {
                        greeting = Greeting::make_greeting(gr.protocol_version,
                            gr.message);
                        if PROTOCOL_VERSION != greeting.protocol_version {
                            println!("Cannot communicate with server -
                                        different versions");
                            return 1
                        }
                        else {
                            println!("Protocol version: {}\n{}",
                                greeting.protocol_version, greeting.message);
                        }
                    },
                    _ => {
                        println!("Could not decode greet-package");
                        return 1
                    }
                }
            }
        },
        Err(_) => return 1
    }
    0
}

/// Read login information from command line and send it to the server
fn send_login<W: Write>(buf: &mut W) -> u8 {
    let mut login = Login::new();
    println!("Username: ");
    login.username = cmd_read();

    println!("\nPassword: ");
    login.password = cmd_read();

    //send Login package to server
    let status = buf.write_u8(Cnv::LoginPkg as u8);
    match status {
        Ok(_) => {
            let encode = encode_into(&login, buf, SizeLimit::Bounded(1024));
            match encode {
                Ok(_) => return 0,
                Err(_) => {
                    println!("Sending login-data failed");
                    return 1
                }
            }
        },
        Err(_) => {
            println!("Sending package header failed");
            return 1
        }
    }
}
