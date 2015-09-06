extern crate docopt;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate server;

use rustc_serialize::json;
use std::fs::File;
use std::io::Read;
use docopt::Docopt;
use std::net::Ipv4Addr;
use std::str::FromStr;

/// For console input, manages flags and arguments
const USAGE: &'static str = "
Usage: uosql-server [--cfg=<file>] [--bind=<address>] [--port=<port>]
[--dir=<directory>]

Options:
    --cfg=<file>        Enter a configuration file.
    --bind=<address>    Change the bind address.
    --port=<port>       Change the port.
    --dir=<directory>   Change the path of the database.
";

#[derive(Debug, RustcDecodable)]
struct Args {
   flag_cfg: Option<String>,
   flag_bind: Option<String>,
   flag_port: Option<u16>,
   flag_dir: Option<String>
}

/// Entry point for server.
fn main() {
    // Configure and enable the logger. We may `unwrap` here, because a panic
    // would happen right after starting the program
    server::logger::with_loglevel(log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();
    info!("Starting uoSQL server...");

    // Getting the information for a possible configuration
    let args : Args = Docopt::new(USAGE).and_then(|d| d.decode())
                                        .unwrap_or_else(|e| e.exit());

    // If a cfg is entered, use this file name to set configurations
    let mut config = read_conf_from_json(args.flag_cfg
                                .unwrap_or("config.json".into()));

    // Change the bind address if flag is set
    config.address = args.flag_bind.and_then(|b| Ipv4Addr::from_str(&b)
                                   .ok()).unwrap_or(config.address);

    // Change port if flag is set
    config.port = args.flag_port.unwrap_or(config.port);

    // Change directory is flag is set
    config.dir = args.flag_dir.unwrap_or(config.dir);

    info!("Bind: {}  Port: {}  Directory: {}",
                        config.address, config.port, config.dir);

    // Start listening for incoming Tcp connections
    server::listen(config);
}


/// Creates a Config struct out of a config file
/// returns default values for everything that is
/// not entered manually
fn read_conf_from_json(name: String) -> server::Config {

    #[derive(Debug, RustcDecodable, Default)]
    struct CfgFile {
        address: Option<String>,
        port: Option<u16>,
        dir: Option<String>
    }

    // Read from JSON file and decode to CfgFile
    let mut config = CfgFile::default();
    if let Ok(mut f) = File::open(name) {
        let mut s = String::new();
        if let Err(e) = f.read_to_string(&mut s) {
            error!("Could not read JSON-file: {:?}", e)
        } else {
            config = json::decode(&s).unwrap();
        }
    }

    let s = config.address.unwrap_or("127.0.0.1".into());
    let bind = match Ipv4Addr::from_str(&s) {
        Ok(n) => n,
        Err(_) => {
            warn!("Invalid bind address, set to default");
            Ipv4Addr::new(127,0,0,1)
        }
    };

    // Return configuration, all None datafields set to default
    server::Config {
        address: bind,
        port: config.port.unwrap_or(4242),
        dir: config.dir.unwrap_or("data".into())
    }
}
