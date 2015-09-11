//! Simple client program
//! Establishes connection to server and sends login information
#[macro_use]
extern crate log;
extern crate uosql;
extern crate bincode;
extern crate byteorder;
extern crate docopt;
extern crate rustc_serialize;
extern crate server;

mod specialcrate;

use std::io::{self, stdout, Write};
use std::str::FromStr;
use uosql::logger;
use uosql::Error;
use uosql::Connection;
use server::storage::Rows;
use docopt::Docopt;
use std::net::Ipv4Addr;
use std::cmp::{max, min};

/// For console input, manages flags and arguments
const USAGE: &'static str = "
Usage: uosql-client [--bind=<address>] [--port=<port>] [--name=<username>]
        [--pwd=<password>]

Options:
    --bind=<address>    Change the bind address.
    --port=<port>       Change the port.
    --name=<username>   Login with given username.
    --pwd=<password>    Login with given password.
";

#[derive(Debug, RustcDecodable)]
struct Args {
   flag_bind: Option<String>,
   flag_port: Option<u16>,
   flag_name: Option<String>,
   flag_pwd:  Option<String>
}

fn main() {

    logger::with_loglevel(log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();

    // Getting the information for a possible configuration
    let args : Args = Docopt::new(USAGE).and_then(|d| d.decode())
                                        .unwrap_or_else(|e| e.exit());

    // Change the bind address if flag is set
    let address = {
        match args.flag_bind {
            Some(a) => {
                if Ipv4Addr::from_str(&a).is_ok() { a }
                else { read_address() }
            },
            None => {
                read_address()
            }
        }
    };

    // Change port if flag is set
    let port = {
        match args.flag_port {
            Some(p) => {
                if p > 1024 {
                    p
                } else {
                    read_port()
                }
            },
            None => read_port()
        }
    };

    // Set username for connection
    let username = {
        match args.flag_name {
            Some(u) => u,
            None => read_string("Username")
        }
    };

    // Set password for connection
    let password = {
        match args.flag_pwd {
            Some(p) => p,
            None => read_string("Password")
        }
    };

    let mut conn = match Connection::connect(address, port, username, password)
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
                },
                Error::Server(e) => {
                    error!("{}", e.msg);
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
        ":hello" => {
            println!("Hello, Dave. You're looking well today.");
        }
        ":snake" => {
            println!("Not on a plane, but on your terminal");
            println!("Thanks for Snake-Code (MIT License) to Johannes Schickling
                    via github /schickling/rust-examples/tree/master/snake-ncurses");
            specialcrate::snake();
        }
        _ => {

            // Size
            match conn.execute(input.into()) {
                Ok(data) => {
                    // show data belonging to executed query
                    display(&data);
                },
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
                        Error::Server(e) => {
                            error!("{}", e.msg);
                            return true
                        }
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

/// Read from command line and return trimmed string
/// If an error occurs reading from stdin loop until a valid String was read
fn read_line() -> String {
    let mut input = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                match &*input {
                    "\n" => return input,
                    _ => return input.trim().into()
                }
            },
            _ => { }
        }
    }
}

/// Read IP-address to connect to from command-line
/// In case no input given ("\n") the default address "127.0.0.1" is returned
pub fn read_address() -> String {
    loop {
        print!("IP: ");
        let e = stdout().flush();
        match e {
            Ok(_) => {},
            Err(_) => info!("")
        }
        let a = read_line();
        match &*a {
            "\n" => return "127.0.0.1".into(),
            _ => {
                if Ipv4Addr::from_str(&a).is_ok() {
                    return a
                }
            }
        }
    }
}

/// Read Port number to connect to from command-line
/// In case no input given ("\n") the default port "4242" is returned
pub fn read_port() -> u16 {
    loop {
        print!("Port: ");
        let e = stdout().flush();
        match e {
            Ok(_) => {},
            Err(_) => info!("")
        }
        let a = read_line();
        match &*a {
            "\n" => return 4242,
            _ => {
                let p: Option<u16> = a.trim().parse::<u16>().ok();
                match p {
                    Some(p) => {
                        if p > 1024 {
                            return p
                        } else {
                            warn!("Valid port range: 1024 < port <= 65535")
                        }
                    },
                    None => {}
                }
            }
        }
    }
}

pub fn read_string(msg: &str) -> String {
    loop {
        print!("{}: ", msg);
        let e = stdout().flush();
        match e {
            Ok(_) => {},
            Err(_) => info!("")
        }
        let r = read_line();
        match &*r {
            "\n" => {},
            _ => return r.trim().to_string()
        }
    }
}

pub fn display(row: &Rows) {
    if row.data.is_empty() {
        display_meta(&row)
    } else {
        display_data(&row)
    }
}

fn display_data(row: &Rows) {
    let mut cols = vec![];
    for i in &row.columns {
        cols.push(max(12, i.name.len()));
    }

    // column names
    display_seperator(&cols);

    for i in 0..(cols.len()) {
        if row.columns[i].name.len() > 27 {
            print!("| {}... ", &row.columns[i].name[..27]);
        } else {
            print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].name);
        }
    }
    println!("|");

    display_seperator(&cols);

}

fn display_meta(row: &Rows) {
    // print meta data
    let mut cols = vec![];
    for i in &row.columns {
        cols.push(max(12, max(i.name.len(), i.description.len())));
    }

    // Column name +---
    print!("+");
    let col_name = "Column name";
    for _ in 0..(col_name.len()+2) {
        print!("-");
    }

    // for every column +---
    display_seperator(&cols);

    print!("| {} ", col_name);
    // name of every column
    for i in 0..(cols.len()) {
        if row.columns[i].name.len() > 27 {
            print!("| {}... ", &row.columns[i].name[..27]);
        } else {
            print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].name);
        }
    }
    println!("|");

    // format +--
    print!("+");
    for _ in 0..(col_name.len()+2) {
        print!("-");
    }

    display_seperator(&cols);

    print!("| {1: <0$} ", col_name.len(), "Type");
    for i in 0..(cols.len()) {
        print!("| {1: ^0$} ", min(30, cols[i]), format!("{:?}", row.columns[i].sql_type));
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Primary");
    for i in 0..(cols.len()) {
        print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].is_primary_key);
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Allow NULL");
    for i in 0..(cols.len()) {
        print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].allow_null);
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Description");
    for i in 0..(cols.len()) {
        if row.columns[i].description.len() > 27 {
            //splitten
            print!("| {}... ", &row.columns[i].description[..27]);
        } else {
            print!("FALSE");
            print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].description);
        }
    }
    println!("|");

    print!("+");
    for _ in 0..(col_name.len()+2) {
        print!("-");
    }

    display_seperator(&cols);
}

pub fn display_seperator(cols: &Vec<usize>) {
    for i in 0..(cols.len()) {
        print!("+--");
        for _ in 0..min(30, cols[i]) {
            print!("-");
        }
    }
    println!("+");
}
