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
#![allow(dead_code, unused_variables)]
use std::net::TcpStream;
use std::io::{Write,Read, Error, ErrorKind};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt}; // for write_u16()
use std::io;
use bincode::rustc_serialize::{decode_from, encode_into};
use bincode::SizeLimit;
use std::io::Cursor;

#[derive(RustcEncodable, RustcDecodable)]
#[repr(u8)]
pub enum CNV{
	GREETING = 0,
	LOGIN,
	ERROR,
}
#[derive(RustcEncodable, RustcDecodable)]
#[repr(u8)]
pub enum ErrorCode{
	UNSPECIFIC_ERR = 0,
	IO_ERR,
	UNEXPEC_PKG_ERR,
}

const PROTOCOL_VERSION : u8 = 1;

/// This is the first packet being sent by the server after the TCP connection
/// is established.
#[derive(RustcEncodable, RustcDecodable)]
pub struct Greeting {
    protocol_version: u8,	// 1 byte
    message: String,		// n bytes
}

impl Greeting{
	pub fn make_greeting(version: u8, msg: String)-> Greeting{
		Greeting{protocol_version: version, message: msg}
	}
}

/// writes a welcome-message to the given server-client-stream
pub fn do_handshake<W:Write + Read>(stream: &mut W) -> Result<(String, String), io::Error>{
	let greet = Greeting::make_greeting(PROTOCOL_VERSION, "Welcome".to_string());
	
	// send handshake package to client
	stream.write(&[CNV::GREETING as u8]); //kind of message
	let greet_encode = encode_into(&greet, stream, SizeLimit::Bounded(1024));

	// receive login data from client
	let mut login = Login::new();
	match read_login(stream, &mut login){
		Ok(something) => return Ok((login.username, login.password)),
		Err(msg) => return Err(msg)
	}
}

/// The client responds with this packet to a `Greeting` packet, finishing the
/// authentication handshake.
#[derive(RustcEncodable, RustcDecodable)]
pub struct Login{
	username: String,
	password: String
}

impl Login{
	// default values
	pub fn new() -> Login{
		Login{username: "".to_string(), password: "".to_string()}
	}

	pub fn set_name(&mut self, usern: String){
		self.username = usern
	}

	pub fn set_password(&mut self, passwd: String){
		self.password = passwd
	}
}

/// reads the data from the response to the handshake,
/// username and password extracted and authenticated
pub fn read_login<R:Read+Write>(stream: &mut R, login: &mut Login) -> Result<(), io::Error>{
	
	// read the first byte
	let status = try!(stream.read_u8());
	if status != CNV::LOGIN as u8 {
		//send error_package
		send_error_package(stream, ErrorCode::UNEXPEC_PKG_ERR);
		return Ok(())
	}

	let res = decode_from::<R,Login>(stream, SizeLimit::Bounded(1024));
	match res{
		Ok(log) => {login.set_name(log.username); 
			login.set_password(log.password); 
			return Ok(())},
		_=> Err(Error::new(ErrorKind::Other, "not again"))
	}
}

/// read the given number of bytes, returns a vector of bytes else io::Error 
pub fn read_bytes<R:Read>(stream: &mut R, len: u16)-> Result<Vec<u8>, io::Error>{
	let mut vec = vec![];			// vector to store the bytes
	let mut chunk = stream.take(len as u64);	// new TakeReader to read the len bytes

	let status = chunk.read_to_end(&mut vec); // reads the bytes and stores it in vec

	match status{
		Ok(n)=> if len as usize == n {return Ok(vec)} else {return Err(Error::new(ErrorKind::Other, "eof"))},
		Err(msg)=> return Err(msg)

	}
}

pub fn send_error_package<W:Write>(stream: &mut W, err: ErrorCode){
	stream.write(&[CNV::ERROR as u8]);
	stream.write(&[err as u8]);
}

/// Sent by the client to the server.
///
/// Many commands are executed via query, but there are some "special"
/// commands that are not sent as query.
pub enum Command {
    Quit,
    Ping,
    Query(String),
    // Shutdown,
    // Statistics,
}

/// Sent by the server to the client.
pub struct Response;    // TODO


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
pub fn testlogin(){
	let mut vec = Vec::new();

	//original 
	let mut login = Login::new();
	login.set_name("elena".to_string());
	login.set_password("praktikum".to_string());

	let mut login_res = Login::new();

	vec.push(1u8);
	let login_encode = encode_into(&login,&mut vec,SizeLimit::Bounded(1024));

	
	let login_decode = read_login(&mut Cursor::new(vec), &mut login_res);
	assert_eq!(login_res.username, "elena");
	assert_eq!(login.password, "praktikum");
}