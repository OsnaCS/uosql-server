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

//constants for message status
enum CNV{
	GREETING,
	LOGIN,
}
/***********************************/

const PROTOCOL_VERSION : u8 = 1;

/// trait to pack the messages for sending
pub trait ToNetwork{ // TODO: set trait private 
	fn write<W:Write>(&self, w: &mut W);
}

/// This is the first packet being sent by the server after the TCP connection
/// is established.
pub struct Greeting {// TODO set private 
    pub protocol_version: u8,	// 1 byte
    pub size_of_message: u16,	// 2 bytes
    pub message: String,		// n bytes
}

impl Greeting{
	pub fn make_greeting(version: u8, msg: String)-> Greeting{
		Greeting{protocol_version: version, size_of_message: msg.len() as u16, message: msg}
	}
}

/// implementation of the write method for the struct greeting 
impl ToNetwork for Greeting{
	fn write<W:Write>(&self, w: &mut W){
		w.write(&[CNV::GREETING as u8]); //kind of message
		w.write(&[self.protocol_version]);
		w.write_u16::<BigEndian>(self.size_of_message).unwrap();
		w.write(self.message.as_bytes());
	}
}

/// writes a welcome-message to the given server-client-stream
pub fn do_handshake<W:Write + Read>(stream: &mut W) -> Result<(String, String), io::Error>{
	let greet = Greeting::make_greeting(PROTOCOL_VERSION, "Welcome".to_string());
	greet.write(stream);

	let mut login = Login::new();
	match read_login(stream, &mut login){
		Ok(something) => return Ok((login.username, login.password)),
		Err(msg) => return Err(msg)
	}

}

/// The client responds with this packet to a `Greeting` packet, finishing the
/// authentication handshake.
pub struct Login{
	username: String,
	password: String
}

impl Login{
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
pub fn read_login<R:Read>(stream: &mut R, login: &mut Login) -> Result<(), io::Error>{
	// read the first byte
	let status = try!(stream.read_u8());
	if status != CNV::LOGIN as u8 {
		//set error_package
		// TODO
		return Ok(())
	}
	//read the lengths of username and passwords
	let username_len = try!(stream.read_u16::<BigEndian>());
	let password_len = try!(stream.read_u16::<BigEndian>());
	
	// read the bytes for username and password
	let mut vec_username = try!(read_bytes(stream, username_len));
	let mut vec_password = try!(read_bytes(stream, username_len));

	// convert bytes to strings
	login.set_name(String::from_utf8(vec_username).unwrap());
	login.set_password(String::from_utf8(vec_password).unwrap());

	Ok(())
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
