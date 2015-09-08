#[macro_use]
extern crate server;
extern crate bincode;

use std::net::{Ipv4Addr, SocketAddrV4, AddrParseError, TcpStream};
use std::str::FromStr;
use std::io::{self, Write, Read};
pub use server::net::types::{Command, PkgType};
pub use server::logger;
use bincode::SizeLimit;
use bincode::rustc_serialize::{EncodingError, DecodingError, decode_from, encode_into};

pub enum Error {
    AddrParse(AddrParseError),
    Io(io::Error),
    UnexpectedPkg(String),
    Encode(EncodingError),
    Decode(DecodingError),
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
    ip: Ipv4Addr,
    port: u16,
    tcp: TcpStream,
}

impl Connection {
    /// Establish connection to specified address and port
    pub fn connect(addr: String, port: u16) -> Result<Connection, Error> {
        let tmp_addr = match std::net::Ipv4Addr::from_str(&addr) {
            Ok(tmp_addr) => tmp_addr,
            Err(e) => return Err(Error::AddrParse(e))
        };
        match TcpStream::connect((tmp_addr, port)) {
            Ok(tcp) => Ok(Connection { ip: tmp_addr, port: port, tcp: tcp } ),
            Err(e) => Err(Error::Io(e))
        }
    }

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
}

fn send_cmd<W: Write>(mut s: &mut W, cmd: Command, size: u64) -> Result<(), Error> {
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
