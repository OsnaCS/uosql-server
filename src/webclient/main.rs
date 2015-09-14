#[macro_use]
extern crate log;
#[macro_use]
extern crate nickel;
extern crate plugin;
extern crate typemap;
extern crate hyper;
extern crate uosql;
extern crate rustc_serialize;
extern crate cookie;
extern crate url;

use uosql::Connection;
use std::io::Read;
use uosql::Error;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use plugin::Extensible;
use hyper::header::{Cookie, SetCookie};
use nickel::{Nickel, HttpRouter};
use cookie::Cookie as CookiePair;
use hyper::method::Method;
use url::form_urlencoded as urlencode;
use std::net::Ipv4Addr;
use std::str::FromStr;
use nickel::QueryString;

// Dummy key for typemap
struct ConnKey;
impl typemap::Key for ConnKey {
    type Value = Arc<Mutex<Connection>>;
}

#[derive(Debug)]
struct Login {
    user : String,
    password: String
}

/// Web based client
fn main() {

    let mut server = Nickel::new();
    let map: HashMap<String, Arc<Mutex<Connection>>>= HashMap::new();
    let map = Arc::new(Mutex::new(map));
    let map2 = map.clone();

    // Cookie managing
    server.utilize(middleware! { |req, res|

        // If login data has been posted, continue
        if req.origin.method == Method::Post {
            return Ok(nickel::Action::Continue(res));
        }

        // Look for session string in Cookies
        let sess = match req.origin.headers.get::<Cookie>() {
            // If no Cookie found, go to Login
            None => {
                let m = HashMap::<i8, i8>::new();
                return res.render("src/webclient/templates/login.tpl", &m);
            }
            // If there is a Cookie, eat it
            // (or find the matching UosqlDB-Cookie and extract session string)
            Some(cs) => {
                if let Some(sess) = cs.to_cookie_jar(&[1u8]).find("UosqlDB") {
                    sess.value
                // There is a cookie, but it is not ours :'(
                // Return to Login
                } else {
                    let m = HashMap::<i8, i8>::new();
                    return res.render("src/webclient/templates/login.tpl", &m);
                }
            },
        };

        // We have a session string and look for the matching connection in
        // our Session-Connection map
        let guard = map.lock().unwrap();
        match guard.get(&sess) {
            // No matching session: Old cookie
            None => {
                let mut data = HashMap::new();
                data.insert("err_msg", "Invalid Session");
                return res.render("src/webclient/templates/login.tpl", &data);
            }
            // There is a connection, we are logged in, we can enter the site!
            Some(con) => {
                req.extensions_mut().insert::<ConnKey>(con.clone());
                return Ok(nickel::Action::Continue(res));
            }
        }
    });

    // Login managing
    server.post("/login", middleware! { |req, mut res|

        // Read the post data
        let mut login_data = String::new();
        let read = req.origin.read_to_string(&mut login_data).unwrap();

        // Not sufficiently filled in, return to Login with error msg
        if read < 15 {
            let mut data = HashMap::new();
            data.insert("err_msg", "No data given");
            return res.render("src/webclient/templates/login.tpl", &data);
        }

        // Extract login data from Post string
        let pairs = urlencode::parse(login_data.as_bytes());
        let username = pairs.iter().find(|e| e.0 == "user").map(|e| e.1.clone());
        let password = pairs.iter().find(|e| e.0 == "password").map(|e| e.1.clone());
        let bind_in = pairs.iter().find(|e| e.0 == "bind").map(|e| e.1.clone());
        let port_in = pairs.iter().find(|e| e.0 == "port").map(|e| e.1.clone());

        // If eihter username or password are empty, return to Login page
        if username.is_none() || password.is_none()  {
            let mut data = HashMap::new();
            data.insert("err_msg", "Not all required fields given");
            return res.render("src/webclient/templates/login.tpl", &data);
        }

        let mut connection = "127.0.0.1".to_string();
        // Bind_in is never none, for inexplicable reasons
        if bind_in.clone().unwrap().len() > 8 {
            connection = bind_in.unwrap();
            test_bind(&connection);
        }

        let port = port_in.unwrap_or("4242".into()).parse::<u16>().unwrap_or(4242);

        // build Login struct
        let login = Login {
            user: username.unwrap(),
            password: password.unwrap()
        };

        // Generate new session string
        let sess_str = login.user.clone(); // Dummy

        // Try connect to db server
        // Insert connection and session string into hashmap
        let mut guard = map2.lock().unwrap();

        // create new connections
        match guard.deref_mut().entry(sess_str.clone()) {
            Entry::Occupied(_) => {},
            Entry::Vacant(v) => {
                let cres = Connection::connect(connection, port,
                                               login.user.clone(), login.password.clone());
                match cres {
                    Err(e) => {
                        let errstr = match e {
                            // Connection error handling
                            // TO DO: Wait for Display/Debug
                            Error::AddrParse(_) => {
                                "Could not connect to specified server."
                            },
                            Error::Io(_) => {
                                "Connection failure. Try again later."
                            },
                            Error::Decode(_) => {
                                "Could not readfsdfd data from server."
                            },
                            Error::Encode(_) => {
                                "Could not send data to server."
                            },
                            Error::UnexpectedPkg => {
                                "Unexpected Package."
                            },
                            Error::Auth => {
                                "Authentication failed."
                            },
                            Error::Server(_) => {
                                "Network Error."
                            },
                        };
                        let mut data = HashMap::new();
                        data.insert("err", errstr);
                        return res.render("src/webclient/templates/error.tpl", &data);
                    }
                    Ok(c) => {
                        v.insert(Arc::new(Mutex::new(c)));
                    },
                }
            }
        };

        // Set a Cookie with the session string as its value
        // sess_str is set to a value here, so we can safely unwrap
        let keks = CookiePair::new("UosqlDB".to_owned(), sess_str.clone());
        res.headers_mut().set(SetCookie(vec![keks]));

        // Redirect to the greeting page
        *res.status_mut() = nickel::status::StatusCode::Found;
        res.headers_mut().set_raw("location", vec![b"/".to_vec()]);
        return res.send("");
    });

    // Disconnect from server
    server.get("/logout", middleware! { |req, mut res|

        let mut con = req.extensions().get::<ConnKey>().unwrap().lock().unwrap();
        let mut data = HashMap::new();

        data.insert("name", con.get_username().to_string());

        con.quit();

        // Remove Cookie
        let sess = match req.origin.headers.get::<Cookie>() {

            None => { }
            Some(cs) => {
                let cj = cs.to_cookie_jar(&[1u8]);
                cj.remove("UosqlDB");
                res.headers_mut().set(SetCookie::from_cookie_jar(&cj));
            },
        };

        return res.render("src/webclient/templates/logout.tpl", &data);
    });

        // Greeting page
    server.get("/", middleware! { |req, response|

        // Look for connection
        let tmp = req.extensions().get::<ConnKey>().unwrap().clone();
        let mut con = tmp.lock().unwrap();

        let query = req.query().get("sql");
        if !query.is_none() {
            println!("{:?}", query.unwrap());
            match con.execute(query.unwrap().to_string()) {
                Ok(m) => println!("{:?}", m),
                Err(e) => {
                    match e {
                        Error::Io(_) => {
                            error!("Connection failure. Try again later.");
                        },
                        Error::Decode(_) => {
                            error!("Could not read data from server.");
                        }
                        Error::Encode(_) => {
                            error!("Could not send data to server.");
                        }
                        Error::UnexpectedPkg => {
                            error!("{}", e.to_string());
                        },
                        Error::Server(e) => {
                            error!("{}", e.msg);
                        }
                        _ => {
                            error!("Unexpected behaviour during execute()");
                        }
                    }
                }
            }
        }

        // Current display with short welcome message
        let version = con.get_version().to_string();
        let port = con.get_port().to_string();
        let mut data = HashMap::new();

        data.insert("name", con.get_username());
        data.insert("version", &version);
        data.insert("bind", con.get_ip());
        data.insert("port", &port);
        data.insert("msg", con.get_message());
        return response.render("src/webclient/templates/main.tpl", &data);
    });

    server.listen("127.0.0.1:6767");
}

fn test_bind (bind : &str) -> bool {
    let result = match Ipv4Addr::from_str(bind) {
        Ok(_) => true,
        Err(_) => {
            false
        }
    };
    result
}
