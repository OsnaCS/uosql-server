//! The network api
//!
//! This module defines types and functions to read and write objects from
//! and to a TcpStream.
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

// TODO: Remove this line as soon as these module is actually used
#![allow(dead_code, unused_variables)]

const PROTOCOL_VERSION : u8 = 1;

/// This is the first packet being sent by the server after the TCP connection
/// is established.
pub struct Greeting {
    protocol_version: u8,

}

/// The client responds with this packet to a `Greeting` packet, finishing the
/// authentication handshake.
pub struct Login;

/// Sent by the client to the server.
pub struct Command;

/// Sent by the server to the client.
pub struct Response;
