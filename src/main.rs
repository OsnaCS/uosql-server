#[macro_use]
extern crate log;
extern crate term_painter as term;
extern crate byteorder;
extern crate rustc_serialize;
extern crate bincode;

use rustc_serialize::json;
use std::fs::File;
use std::env;
use std::io::Read;

pub mod auth;
pub mod conn;
pub mod logger;
pub mod net;
pub mod parse;
pub mod query;
pub mod storage;

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
    let args : Vec <_> = env::args().collect();

    let mut config: Config = read_conf_from_json("src/config.json".to_string());

    println!("{:?}", config);

    // Start listening for incoming Tcp connections
    listen(config);
}

fn listen(config: Config) {
    use std::net::TcpListener;
    use std::thread;

    // Collecting information for binding process
    let mut bind_inf = format!("{}:{}",
        config.address.unwrap_or("127.0.0.1".to_string()),
        config.port.unwrap_or("4242".to_string()));

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
    let mut config = Config::default_Config();
    if let Ok(mut f) = File::open(name) {
            let mut s = String::new();
        if let Err(e) = f.read_to_string(&mut s) {
            println!("Error");
        } else {
            config = json::decode(&s).unwrap();
        }
    }
    config
}

#[derive(Debug, RustcDecodable)]
pub struct Config {
    address: Option<String>,
    port: Option<String>,
    dir: Option<String>
}

impl Config {
    pub fn default_Config() -> Config {
        Config { address : Some("127.0.0.1".to_string()),
        port : Some("4242".to_string()) ,
        dir : Some("/somewhere".to_string())}
    }
}
