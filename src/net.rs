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

const PROTOCOL_VERSION : u8 = 1;

/// This is the first packet being sent by the server after the TCP connection
/// is established.
pub struct Greeting {
    protocol_version: u8,

}

/// The client responds with this packet to a `Greeting` packet, finishing the
/// authentication handshake.
pub struct Login;   // TODO

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
