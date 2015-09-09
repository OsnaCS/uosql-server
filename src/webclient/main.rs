extern crate iron;
extern crate uosql;

use iron::prelude::*;
use iron::status;
use uosql::Connection;
use uosql::Error;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
/// Web based client
fn main() {

    // Key Value pairs: User + connections
    // Every user has one connection
    let map: HashMap<String, Connection> = HashMap::new();
    let ptr = Arc::new(Mutex::new(map));

    let mut chain = Chain::new(move |_: &mut Request| {
        let own = ptr.clone();
        // Locked until guard dies
        let mut guard = own.lock().unwrap();

//==================== TODO Read user data input somehow ==========================//
        // Get login data
        let username = "Peter".to_string();
        let connection = "127.0.0.1".to_string();
        let port = 4242;
        let password = "Bob".to_string();

        // Get connection of user or build new connection and insert it into
        // hashmap
        let con = match guard.deref_mut().entry(username.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let cres = Connection::connect(connection, port,
                                               username, password);
                match cres {
                    Err(e) => {
                        match e {
                            Error::AddrParse(_) => {
                                return Ok(Response::with((iron::status::Ok,
                                format!("Could not connect to specified server."))))
                            },
                            Error::Io(_) => {
                                return Ok(Response::with((iron::status::Ok,
                                format!("Connection failure. Try again later."))))
                            },
                            Error::Decode(_) => {
                                return Ok(Response::with((iron::status::Ok,
                                format!("Could not read data from server."))))
                            },
                            Error::Encode(_) => {
                                return Ok(Response::with((iron::status::Ok,
                                format!("Could not send data to server."))))
                            },
                            Error::UnexpectedPkg(e) => {
                                return Ok(Response::with((iron::status::Ok,
                                          format!("{}", e.to_string()))))
                            },
                            Error::Auth(e) => {
                                return Ok(Response::with((iron::status::Ok,
                                          format!("{}", e.to_string()))))
                            },
                        }
                    }
                    Ok(c) => v.insert(c),
                }
            }
        };

        // Msg to print to web
        let msg = format!("Connected (version {}) to {}:{}\n{}\n",
                           con.get_version(), con.get_ip(),
                           con.get_port(), con.get_message());

        Ok(Response::with((iron::status::Ok, msg)))
    });
    // Build webclient on localhost 3000
    Iron::new(chain).http("localhost:3000").unwrap();
}

/*
    Get data input from user via webclient and
    process it

*/
