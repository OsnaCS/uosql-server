#[macro_use]
extern crate log;
extern crate term_painter as term;
extern crate byteorder;
extern crate rustc_serialize;
extern crate bincode;
extern crate docopt;

use rustc_serialize::json;
use std::fs::File;
use std::env;
use std::io::Read;
use docopt::Docopt;

pub mod auth;
pub mod conn;
pub mod logger;
pub mod net;
pub mod parse;
pub mod query;
pub mod storage;


/// For console input, manages flags and arguments
const USAGE: &'static str = "
Usage: uosql-server [--cfg=<file>]

Options:
    --cfg=<file>   Enter a configuration file
";

#[derive(Debug, RustcDecodable)]
struct Args {
    // flag_cfg: bool,
   flag_cfg: Option<String>
}

/// Entry point for server. Allow dead_code to supress warnings when
/// compiled as a library.
#[allow(dead_code)]
fn main() {
    // Configure and enable the logger. We may `unwrap` here, because a panic
    // would happen right after starting the program
    logger::with_loglevel(log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();
    info!("Starting uoSQL server...");

    // Getting the information for a possible configuration
    let args : Args = Docopt::new(USAGE).and_then(|d| d.decode())
                                        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    // If a cfg is entered, use this file name to set configurations
    let config = read_conf_from_json(args.flag_cfg
                                .unwrap_or("src/config.json".into()));

    println!("{:?}", config); // for debugging

    // Start listening for incoming Tcp connections
    listen(config);
}


/// Listens for incoming TCP streams
fn listen(config: Config) {
    use std::net::TcpListener;
    use std::thread;

    // Collecting information for binding process
    let mut bind_inf = format!("{}:{}", config.address, config.port);

    let listener = TcpListener::bind("127.0.0.1:4242").unwrap();
    //let listener = TcpListener::bind(bind_inf).unwrap();

    // Accept connections and process them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Connection succeeded: Spawn thread and handle
                thread::spawn(move|| {
                    conn::handle(stream)
                });
            },
            Err(e) => {
                // Something went wrong...
                warn!("Failed to accept incoming connection: {:?}", e);
            },
        }
    }
}

/// Creates a Config struct out of a config file
fn read_conf_from_json(name: String) -> Config {
    /// A struct that contains different configuration details with a
    /// default setting
    #[derive(Debug, RustcDecodable, Default)]
    struct CfgFile {
        address: Option<String>,
        port: Option<u16>,
        dir: Option<String>
    }
    let mut config = CfgFile::default();
    if let Ok(mut f) = File::open(name) {
        let mut s = String::new();
        if let Err(e) = f.read_to_string(&mut s) {
            println!("Error");
        } else {
            config = json::decode(&s).unwrap();
        }
    }
    Config {
        address: config.address.unwrap_or("127.0.0.1".into()),
        port: config.port.unwrap_or(4242),
        dir: config.dir.unwrap_or("data".into())
    }
}

#[derive(Debug, RustcDecodable)]
pub struct Config {
    address: String,
    port: u16,
    dir: String
}
