use bincode::rustc_serialize::{EncodingError, DecodingError};
use std::io;
use super::Error;

/// Code numeric value sent as first byte
#[derive(PartialEq, RustcEncodable, RustcDecodable)]
#[repr(u8)]
pub enum PkgType {
    Greet = 0,
    Login,
    Command,
    Error,
    Ok,
    Response,
    AccDenied,
    AccGranted,
}

/// Struct to send the kind of error and error message to the client
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct ClientErrMsg {
    code: u16,
    msg: String
}

/// Convert the possible Error to a serializable ClientErrMsg struct
impl From<Error> for ClientErrMsg {
    fn from(error: Error) -> ClientErrMsg {
        match error {
            Error::Io(_) => ClientErrMsg {
                code: 0,
                msg: "IO error".into()
            },
            Error::UnexpectedPkg(err) => ClientErrMsg {
                code: 2,
                msg: err.into()
            },
            Error::UnknownCmd(err) => ClientErrMsg {
                code: 3,
                msg: err.into()
            },
            Error::Encode(_) => ClientErrMsg {
                code: 4,
                msg: "encoding error".into()
            },
            Error::Decode(_) => ClientErrMsg {
                code: 5,
                msg: "decoding error".into()
            }
        }
    }
}

/// This is the first packet being sent by the server after the TCP connection
/// is established.
#[derive(RustcEncodable, RustcDecodable)]
pub struct Greeting {
    pub protocol_version: u8,   // 1 byte
    pub message: String,        // n bytes
}

impl Greeting {
    pub fn make_greeting(version: u8, msg: String) -> Greeting {
        Greeting { protocol_version: version, message: msg }
    }
}

/// The client responds with this packet to a `Greeting` packet, finishing the
/// authentication handshake.
#[derive(Default, RustcEncodable, RustcDecodable)]
pub struct Login {
    pub username: String,
    pub password: String
}

/// Sent by the client to the server.
///
/// Many commands are executed via query, but there are some "special"
/// commands that are not sent as query.
#[derive(RustcEncodable, RustcDecodable, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    Quit,
    Ping,
    Query(String),
    // Shutdown,
    // Statistics,
}

/// Sent by the server to the client.
pub struct Response;
