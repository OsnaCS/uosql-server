#[macro_use]
extern crate log;
extern crate term_painter as term;

pub mod auth;
pub mod parse;
pub mod storage;
mod logger;

fn main() {
    // Configure and enable the logger. We may `unwrap` here, because a panic
    // would happen right after starting the program
    logger::with_loglevel(log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();
    // info!("Starting uoSQL server...");

    listen();
    // auth::find_user("name", "passwd");
    // println!("Hello, world!");
}


fn listen() {
    trace!("trace trace trace trace trace");
    debug!("debug debug debug debug debug");
    info!("info info info info info");
    warn!("warn warn warn warn warn");
    error!("err err err err err");
}
