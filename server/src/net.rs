//! The network api
//!
//! This module defines types and functions to read and write objects from
//! and to a TcpStream. However, the types in this module are types that are
//! passed to other methods and thus more "high level". They do not represent
//! the byte layout of objects in the protocol!
//!
//! # Protocol
//! All communication is send using TCP, which emulates a data stream. On top
//! of TCP, this database sends single packets.
//!
//! Every packet begins with a four byte `length` field that contains the
//! size of the packet in network byte order.
//!
//! ...
//!

use std::io::{Write, Read};
// to encode and decode the structs to the given stream
use bincode::rustc_serialize::{
    decode_from, encode_into, EncodingError, DecodingError
};
use bincode::SizeLimit;
use rustc_serialize::{Encodable, Encoder}; // to encode the Error
//use byteorder;
use std::io;

const PROTOCOL_VERSION: u8 = 1;

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
            //Error::ByteOrder(_) => ClientErrMsg {
            //    code: 1,
            //    msg: "Byteorder error".into()
            //},
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

/// Collection of possible errors while communicating with the client
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    //ByteOrder(byteorder::Error),
    UnexpectedPkg(String),
    UnknownCmd(String),
    Encode(EncodingError),
    Decode(DecodingError),
}

/// Implement the conversion from byteorder::Error to NetworkError
/*
impl From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Error {
        Error::ByteOrder(err)
    }
}
*/

/// Implement the conversion from io::Error to NetworkError
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
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

/// Write a welcome-message to the given server-client-stream
pub fn do_handshake<W: Write + Read>(stream: &mut W)
    -> Result<(String, String), Error>
{
    let greet = Greeting::make_greeting(PROTOCOL_VERSION, "Welcome".to_string());

    // send handshake packet to client
    try!(encode_into(&PkgType::Greet, stream, SizeLimit::Bounded(1))); //kind of message
    try!(encode_into(&greet, stream, SizeLimit::Bounded(1024)));

    // receive login data from client
    let login = read_login(stream);
    match login {
        Ok(sth) => Ok((sth.username, sth.password)),
        Err(msg) => Err(msg)
    }
}

/// The client responds with this packet to a `Greeting` packet, finishing the
/// authentication handshake.
#[derive(Default, RustcEncodable, RustcDecodable)]
pub struct Login {
    pub username: String,
    pub password: String
}

/// reads the data from the response to the handshake,
/// username and password extracted and authenticated
pub fn read_login<R: Read + Write>(stream: &mut R)
    -> Result<Login, Error>
{
    // read package-type
    let status: PkgType = try!(decode_from(stream, SizeLimit::Bounded(1)));

    if status != PkgType::Login {
        return Err(Error::UnexpectedPkg("package not expected".into()));
    }

    // read the login data
    decode_from(stream, SizeLimit::Bounded(1024)).map_err(|e| e.into())
}


/// send error package with given error code status
pub fn send_error_package<W: Write>(mut stream: &mut W, err: ClientErrMsg)
    -> Result<(), Error>
{
    try!(encode_into(&PkgType::Error, stream, SizeLimit::Bounded(1)));
    try!(encode_into(&err, &mut stream, SizeLimit::Bounded(1024)));
    Ok(())
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

/// Read the sent bytes, extract the kind of command
pub fn read_commands<R: Read + Write>(stream: &mut R)
    -> Result<Command, Error>
{
    // read the first byte for code numeric value
    let status: PkgType = try!(decode_from(stream, SizeLimit::Bounded(1)));
    if status != PkgType::Command {
        //send error_packet
        return Err(Error::UnknownCmd("command not known".into()))
    }

    // second  4 bytes is the kind of command
    decode_from(stream, SizeLimit::Bounded(4096)).map_err(|e| e.into())
}

/// Send error packet with given error code status
pub fn send_error_packet<W: Write>(mut stream: &mut W, err: ClientErrMsg)
    -> Result<(), Error>
{
    try!(encode_into(&PkgType::Error, stream, SizeLimit::Bounded(1)));
    try!(encode_into(&err, &mut stream, SizeLimit::Bounded(1024)));
    Ok(())
}

/// Send ok packet
pub fn send_ok_packet<W: Write>(mut stream: &mut W)
    -> Result<(), Error>
{
    try!(encode_into(&PkgType::Ok, stream, SizeLimit::Bounded(1)));
    Ok(())
}

/// Sent by the server to the client.
pub struct Response;

   // TODO


// # Some information for the `net` working group:
//
// The net module is used by the `conn` module to receive commands from the
// client and to answer those commands.
//
// Your task is to:
// - Design the network protocol, which includes:
//   - What type of data is send when
//   - How to begin a connection
//   - The memory layout of packets
// - Create types that are more "high level" than the byte based network
//   types (see `Command` for example) and that can be used by other modules
// - Implement functions for every step of the connection (handshake,
//   receiving commands, sending answers, ...)
//

#[test]
pub fn test_send_ok_packet() {
    let mut vec = Vec::new();

    let res = send_ok_packet(&mut vec);
    assert_eq!(res.is_ok(), true);
    assert_eq!(vec, vec![PkgType::Ok as u8]);
}

#[test]
pub fn test_send_error_packet() {
    let mut vec = Vec::new();   // stream to write into
    let vec2 = vec![PkgType::Error as u8, // for error packet
        0, 2, // for kind of error
        0, 0, 0, 0, 0, 0, 0, 17, // for the size of the message string
        117, 110, 101, 120, 112, 101, 99, 116, 101, 100, 32, 112, 97, 99, 107, 101, 116];
        // string itself
    let err = Error::UnexpectedPkg("unexpected packet".into());

    // test if the message is sent
    let res = send_error_packet(&mut vec, err.into());
    assert_eq!(res.is_ok(), true);
    assert_eq!(vec, vec2);
}

#[test]
pub fn test_read_commands(){
    // test if the commands are correctly decoded
    use std::io::Cursor;        // stream to read from
    let mut vec = Vec::new();   // stream to write into

    // write the command into the stream
    vec.push(PkgType::Command as u8);
    let _ = encode_into(&Command::Quit, &mut vec, SizeLimit::Bounded(1024));

    // read the command from the stream for Command::Quit
    let mut command_res = read_commands(&mut Cursor::new(vec));
    assert_eq!(command_res.is_ok(), true);
    assert_eq!(command_res.unwrap(), Command::Quit);


    let mut vec2 = Vec::new();
    // write the command into the stream
    vec2.push(PkgType::Command as u8);
    let _ = encode_into(&Command::Query("select".into()),
                                     &mut vec2,
                                     SizeLimit::Bounded(1024));

    // read the command from the stream for Command::Query("select")
    command_res = read_commands(&mut Cursor::new(vec2));
    assert_eq!(command_res.is_ok(), true);
    assert_eq!(command_res.unwrap(), Command::Query("select".into()));
}

#[test]
pub fn testlogin() {
    use std::io::Cursor;        // stream to read from
    let mut vec = Vec::new();   // stream to write into

    // original struct
    let login = Login { username: "elena".into(), password: "praktikum".into() };
    vec.push(1u8);
    let _ = encode_into(&login, &mut vec, SizeLimit::Bounded(1024));

    let login_res = read_login(&mut Cursor::new(vec)).unwrap();

    // test for equality
    assert_eq!(login_res.username, "elena");
    assert_eq!(login_res.password, "praktikum");
}
