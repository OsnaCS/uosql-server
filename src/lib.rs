#[macro_use]
extern crate server;
extern crate bincode;

use std::net::{Ipv4Addr, AddrParseError, TcpStream};
use std::str::FromStr;
use std::io::{self, Write};
pub use server::net::types;
pub use server::logger;
use bincode::SizeLimit;
use bincode::rustc_serialize::{EncodingError, DecodingError,
    decode_from, encode_into};
use types::*;

// const PROTOCOL_VERSION : u8 = 1;

pub enum Error {
    AddrParse(AddrParseError),
    Io(io::Error),
    UnexpectedPkg(String),
    Encode(EncodingError),
    Decode(DecodingError),
    Auth(String),
}

/// Implement the conversion from io::Error to Connection-Error
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

/// Implement the conversion from AddrParseError to Connection-Error
impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Error {
        Error::AddrParse(err)
    }
}

/// Implement the conversion from EncodingError to NetworkError
impl  From<EncodingError> for Error {
    fn from(err: EncodingError) -> Error {
        Error::Encode(err)
    }
}

/// Implement the conversion from DecodingError to NetworkError
impl From<DecodingError> for Error {
    fn from(err: DecodingError) -> Error {
        Error::Decode(err)
    }
}

pub struct Connection {
    ip: String,
    port: u16,
    tcp: TcpStream,
    greeting: Greeting,
    user_data: Login,
}

impl Connection {
    /// Establish connection to specified address and port
    pub fn connect(addr: String, port: u16, usern: String, passwd: String)
        -> Result<Connection, Error>
    {
        // Parse IPv4 address from String
        let tmp_addr = match std::net::Ipv4Addr::from_str(&addr) {
            Ok(tmp_addr) => tmp_addr,
            Err(e) => return Err(e.into())
        };

        // Establish Tcp connection
        let mut tmp_tcp = match TcpStream::connect((tmp_addr, port)) {
            Ok(tmp_tcp) => tmp_tcp,
            Err(e) => return Err(e.into())
        };

        // Greeting message
        match receive(&mut tmp_tcp, PkgType::Greet) {
            Ok(_) => {},
            Err(e) => return Err(e)
        };
        let greet: Greeting =
            try!(decode_from(&mut tmp_tcp, SizeLimit::Bounded(1024)));

        // Login
        let log = Login { username: usern, password: passwd };
        match encode_into(&PkgType::Login, &mut tmp_tcp,
            SizeLimit::Bounded(1024))
        {
            Ok(_) => {},
            Err(e) => return Err(e.into())
        }

        match encode_into(&log, &mut tmp_tcp, SizeLimit::Bounded(1024)) {
            Ok(_) => {},
            Err(e) => return Err(e.into())
        }

        let status: PkgType =
            try!(decode_from(&mut tmp_tcp, SizeLimit::Bounded(1024)));
        match status {
            PkgType::AccGranted =>
                Ok(Connection { ip: addr, port: port, tcp: tmp_tcp,
                    greeting: greet, user_data: log} ),
            PkgType::AccDenied =>
                Err(Error::Auth("Authentication failure.".into())),
            _ => Err(Error::UnexpectedPkg("Unexpected package
                    received. Server is INSANE.".into()))
        }
    }

    /// Sends ping-command to server and receives Ok-package
    pub fn ping(&mut self) -> Result<(), Error> {
        match send_cmd(&mut self.tcp, Command::Ping, 1024) {
            Ok(_) => {},
            Err(e) => return Err(e)
        };
        match receive(&mut self.tcp, PkgType::Ok) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    /// Sends quit-command to server and receives Ok-package
    pub fn quit(&mut self) -> Result<(), Error> {
        match send_cmd(&mut self.tcp, Command::Quit, 1024) {
            Ok(_) => {},
            Err(e) => return Err(e)
        };
        match receive(&mut self.tcp, PkgType::Ok) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    // TODO: Return results (response-package)
    pub fn execute(&mut self, query: String) -> Result<(), Error> {
        match send_cmd(&mut self.tcp, Command::Query(query), 1024) {
            Ok(_) => {},
            Err(e) => return Err(e)
        };
        match receive(&mut self.tcp, PkgType::Ok) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    pub fn get_version(&self) -> u8 {
        self.greeting.protocol_version
    }

    pub fn get_message(&self) -> &str {
        &self.greeting.message
    }

    pub fn get_ip(&self) -> &str {
        &self.ip
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_username(&self) -> &str {
        &self.user_data.username
    }
}

fn send_cmd<W: Write>(mut s: &mut W, cmd: Command, size: u64)
    -> Result<(), Error>
{
    try!(encode_into(&PkgType::Command, s, SizeLimit::Bounded(1024)));
    try!(encode_into(&cmd, &mut s, SizeLimit::Bounded(size)));
    Ok(())
}

/// Match received packages to expected packages
fn receive(s: &mut TcpStream, cmd: PkgType) -> Result<(), Error> {
    let status: PkgType = try!(decode_from(s, SizeLimit::Bounded(1024)));

    if status != cmd {
        return Err(Error::UnexpectedPkg("Received
            unexpected package".into()))
    }
    Ok(())
}

#[test]
fn it_works() {}
