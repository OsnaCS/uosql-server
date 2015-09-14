/// Because of cyclic references to modules we need to use super::Error to use
/// the enum. Nightly Build supports using enums - so we can fix super::Error in
/// about 3 months ;)

use storage::types::Column;
use std::error::Error;

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
    pub msg: String
}

/// Convert the possible Error to a serializable ClientErrMsg struct
impl From<super::Error> for ClientErrMsg {
    fn from(error: super::Error) -> ClientErrMsg {
        match error {
            super::Error::Io(_) => ClientErrMsg {
                code: 0,
                msg: error.description().into()
            },
            super::Error::UnexpectedPkg => ClientErrMsg {
                code: 2,
                msg: error.description().into()
            },
            super::Error::UnknownCmd => ClientErrMsg {
                code: 3,
                msg: error.description().into()
            },
            super::Error::Encode(_) => ClientErrMsg {
                code: 4,
                msg: error.description().into()
            },
            super::Error::Decode(_) => ClientErrMsg {
                code: 5,
                msg: error.description().into()
            },
            super::Error::UnEoq(_) => ClientErrMsg {
                code: 6,
                msg: error.description().into()
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
pub struct Response {
    columns: Vec<Column>,
    data: Option<Vec<u8>>
}
/*
impl ResultSet {
    pub fn get_col_cnt(&self) -> usize {
        self.colums.len()
    }

    pub fn get_col(&self, nr: usize) -> Option<Column> {
        self.columns.get(nr)
    }

    pub fn get_name(&self, name: String)


}
*/
