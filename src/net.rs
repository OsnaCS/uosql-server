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

// TODO: Remove this line as soon as this module is actually used
//#![allow(dead_code, unused_variables)]
use std::io::{Write,Read};
use byteorder::{ReadBytesExt, WriteBytesExt}; // for write_u16()
use bincode::rustc_serialize::{decode_from, encode_into,EncodingError,DecodingError}; // to encode and decode the structs to the given stream
use bincode::SizeLimit;
use rustc_serialize::{Encodable, Encoder}; // to encode the Networkerrors
use byteorder;
use std::io;

const PROTOCOL_VERSION: u8 = 1;

/// Code numeric value sent as first byte
#[derive(RustcEncodable, RustcDecodable)]
#[repr(u8)]
pub enum Cnv {
    GreetPkg = 0,
    LoginPkg,
    CommandPkg,
    ErrorPkg,
    OkPkg,
    ResponsePkg,
    AccDeniedPkg,
    AccGrantedPkg,
}

/// Struct to send the kind of error and error message to the client
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct ClientErrMsg {
    code: u16,
    msg: String
}

/// Convert the possible NetworkErrors to a serializable ClientErrMsg struct
impl From<NetworkErrors> for ClientErrMsg {
    fn from(error: NetworkErrors) -> ClientErrMsg {
        match error {
            NetworkErrors::IoError(_) => ClientErrMsg { code: 0, msg: "IO error".into() },
            NetworkErrors::ByteOrder(_) => ClientErrMsg { code: 1, msg: "Byteorder error".into() },
            NetworkErrors::UnexpectedPkg(err) => ClientErrMsg { code: 2, msg: err.into() },
            NetworkErrors::UnknownCmd(err) => ClientErrMsg { code: 3, msg: err.into() },
            NetworkErrors::EncodeErr(_) => ClientErrMsg { code: 4, msg: "encoding error".into() },
            NetworkErrors::DecodeErr(_) => ClientErrMsg { code: 5, msg: "decoding error".into() }
        }
    }
}

/// Collection of possible errors while communicating with the client
#[derive(Debug)]
pub enum NetworkErrors {
    IoError(io::Error),
    ByteOrder(byteorder::Error),
    UnexpectedPkg(String),
    UnknownCmd(String),
    EncodeErr(EncodingError),
    DecodeErr(DecodingError),
}

/// Implement the conversion from byteorder::Error to NetworkError
impl From<byteorder::Error> for NetworkErrors {
    fn from(err: byteorder::Error) -> NetworkErrors {
        NetworkErrors::ByteOrder(err)
    }
}

/// Implement the conversion from io::Error to NetworkError
impl From<io::Error> for NetworkErrors {
    fn from(err: io::Error) -> NetworkErrors {
        NetworkErrors::IoError(err)
    }
}

/// Implement the conversion from EncodingError to NetworkError
impl  From<EncodingError> for NetworkErrors {
    fn from(err: EncodingError) -> NetworkErrors {
        NetworkErrors::EncodeErr(err)
    }
}

/// Implement the conversion from DecodingError to NetworkError
impl From<DecodingError> for NetworkErrors {
    fn from(err: DecodingError) -> NetworkErrors {
        NetworkErrors::DecodeErr(err)
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
    pub fn make_greeting(version: u8, msg: String)-> Greeting {
        Greeting { protocol_version: version, message: msg }
    }
}

/// Write a welcome-message to the given server-client-stream
pub fn do_handshake<W: Write + Read>(stream: &mut W)
    -> Result<(String, String), NetworkErrors>
{
    let greet = Greeting::make_greeting(PROTOCOL_VERSION, "Welcome".to_string());

    // send handshake packet to client
    try!(stream.write_u8(Cnv::GreetPkg as u8)); //kind of message
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
#[derive(RustcEncodable, RustcDecodable)]
pub struct Login {
    pub username: String,
    pub password: String
}

impl Login {
    // default values
    pub fn new() -> Login {
        Login { username: "".to_string(), password: "".to_string() }
    }
}

/// reads the data from the response to the handshake,
/// username and password extracted and authenticated
pub fn read_login<R: Read+Write>(stream: &mut R)
    -> Result<Login, NetworkErrors>
{
    // read the first byte
    let status = try!(stream.read_u8());
    if status != Cnv::LoginPkg as u8 {
        return Err(NetworkErrors::UnexpectedPkg("package not expected".into()));
    }

    // read the login data
    let res = decode_from(stream, SizeLimit::Bounded(1024));
    match res {
        Ok(log) => Ok(log),
        Err(e) => Err(e.into())
    }
}


/// send error package with given error code status
pub fn send_error_package<W: Write>(mut stream: &mut W, err: ClientErrMsg)
    -> Result<(), NetworkErrors>
{
    try!(stream.write_u8(Cnv::ErrorPkg as u8));
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
    -> Result<Command, NetworkErrors>
{
    // read the first byte for code numeric value
    let status = try!(stream.read_u8());
    if status != Cnv::CommandPkg as u8 {
        //send error_packet
        return Err(NetworkErrors::UnknownCmd("command not known".into()))
    }

    // second  4 bytes is the kind of command
    let command_decode = decode_from(stream, SizeLimit::Bounded(4096));
    match command_decode {
        Ok(command) => Ok(command),
        Err(e) => Err(e.into())
    }
}

/// Send error packet with given error code status
pub fn send_error_packet<W: Write>(mut stream: &mut W, err: ClientErrMsg)
    -> Result<(), NetworkErrors>
{
    try!(stream.write_u8(Cnv::ErrorPkg as u8));
    try!(encode_into(&err, &mut stream, SizeLimit::Bounded(1024)));
    Ok(())
}

/// Send ok packet
pub fn send_ok_packet<W: Write> (mut stream: &mut W)
    -> Result<(), NetworkErrors>
{
    try!(stream.write_u8(Cnv::OkPkg as u8));
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
    assert_eq!(vec, vec![Cnv::OkPkg as u8]);
}

#[test]
pub fn test_send_error_packet() {
    let mut vec = Vec::new();   // stream to write into
    let vec2 = vec![Cnv::ErrorPkg as u8, //for error packet
                        0, 2, // for kind of error
                        0, 0, 0, 0, 0, 0, 0, 17, // for the size of the message string
                        117, 110, 101, 120, 112, 101, 99, 116, 101, 100, 32, 112, 97, 99, 107, 101, 116];
                        // string itself
    let err = NetworkErrors::UnexpectedPkg("unexpected packet".into());

    //test if the message is sent
    let res = send_error_packet(&mut vec, err.into());
    assert_eq!(res.is_ok(), true);
    assert_eq!(vec, vec2);
}

#[test]
pub fn test_read_commands(){
    //test if the commands are correctly decoded
    use std::io::Cursor;        // stream to read from
    let mut vec = Vec::new();   // stream to write into

    // write the command into the stream
    vec.push(Cnv::CommandPkg as u8);
    let command_encode = encode_into(&Command::Quit, &mut vec, SizeLimit::Bounded(1024));

    // read the command from the stream for Command::Quit
    let mut command_res = read_commands(&mut Cursor::new(vec));
    assert_eq!(command_res.is_ok(), true);
    assert_eq!(command_res.unwrap(), Command::Quit);


    let mut vec2 = Vec::new();
    // write the command into the stream
    vec2.push(Cnv::CommandPkg as u8);
    let command_encode = encode_into(&Command::Query("select".into()), &mut vec2, SizeLimit::Bounded(1024));

    // read the command from the stream for Command::Query("select")
    command_res = read_commands(&mut Cursor::new(vec2));
    assert_eq!(command_res.is_ok(), true);
    assert_eq!(command_res.unwrap(), Command::Query("select".into()));
}

#[test]
pub fn testlogin() {
    use std::io::Cursor;        // stream to read from
    let mut vec = Vec::new();   // stream to write into

    //original struct
    let login = Login { username: "elena".into(), password: "praktikum".into() };
    vec.push(1u8);
    let login_encode = encode_into(&login,&mut vec,SizeLimit::Bounded(1024));

    let login_res = read_login(&mut Cursor::new(vec)).unwrap();

    // test for equality
    assert_eq!(login_res.username, "elena");
    assert_eq!(login_res.password, "praktikum");
}
