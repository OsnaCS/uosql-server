#[macro_use]
extern crate log;
extern crate term_painter as term;

pub mod auth;
pub mod parse;
pub mod storage;
pub mod conn;
mod logger;

fn main() {
    // Configure and enable the logger. We may `unwrap` here, because a panic
    // would happen right after starting the program
    logger::with_loglevel(log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();
    info!("Starting uoSQL server...");

    listen();
    // auth::find_user("wwacker", "123456");
}

fn listen() {
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:4242").unwrap();

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
