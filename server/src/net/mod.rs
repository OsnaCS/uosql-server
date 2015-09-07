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
pub mod types;

use std::io::{Write, Read};
// to encode and decode the structs to the given stream
use bincode::rustc_serialize::{decode_from, encode_into};
use bincode::SizeLimit;
use net::types::*;

const PROTOCOL_VERSION: u8 = 1;

/// Write a welcome-message to the given server-client-stream
pub fn do_handshake<W: Write + Read>(stream: &mut W)
    -> Result<(String, String), Error>
{
    let greet = Greeting::make_greeting(PROTOCOL_VERSION, "Welcome".into());

    // send handshake packet to client
    try!(encode_into(&PkgType::Greet, stream, SizeLimit::Bounded(1024))); //kind of message
    try!(encode_into(&greet, stream, SizeLimit::Bounded(1024)));

    // receive login data from client
    let login = read_login(stream);
    match login {
        Ok(sth) => Ok((sth.username, sth.password)),
        Err(msg) => Err(msg)
    }
}

/// reads the data from the response to the handshake,
/// username and password extracted and authenticated
pub fn read_login<R: Read + Write>(stream: &mut R)
    -> Result<Login, Error>
{
    // read package-type
    let status: PkgType = try!(decode_from(stream, SizeLimit::Bounded(1024)));

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
    try!(encode_into(&PkgType::Error, stream, SizeLimit::Bounded(1024)));
    try!(encode_into(&err, &mut stream, SizeLimit::Bounded(1024)));
    Ok(())
}

/// Read the sent bytes, extract the kind of command
pub fn read_commands<R: Read + Write>(stream: &mut R)
    -> Result<Command, Error>
{
    // read the first byte for code numeric value
    let status: PkgType = try!(decode_from(stream, SizeLimit::Bounded(1024)));
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
    try!(encode_into(&PkgType::Error, stream, SizeLimit::Bounded(1024)));
    try!(encode_into(&err, &mut stream, SizeLimit::Bounded(1024)));
    Ok(())
}

/// Send ok packet
pub fn send_ok_packet<W: Write>(mut stream: &mut W)
    -> Result<(), Error>
{
    try!(encode_into(&PkgType::Ok, stream, SizeLimit::Bounded(1024)));
    Ok(())
}

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
