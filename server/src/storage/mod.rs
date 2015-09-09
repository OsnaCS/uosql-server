//! Storage Engine trait and several implementations
//!
//!
pub mod engine;
mod meta;
pub mod types;

mod data;

pub use self::meta::Table;
pub use self::meta::Database;
pub use self::data::Rows;
pub use self::types::Column;
pub use self::types::SqlType;
use parse::ast;
use std::string::FromUtf8Error;

use std::io;

//use self::meta::Table;

use bincode::rustc_serialize::{EncodingError, DecodingError};

/// A database table
///
/// Through this type, you can retreive certain meta information about the
/// table (like column names, column types, storage engine, ...). It's `access`
/// method locks the table globally and returns a storage engine to access
/// the table data.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    BinEn(EncodingError),
    BinDe(DecodingError),
    Byteorder(::byteorder::Error),
    Utf8Error(FromUtf8Error),
    WrongMagicNmbr,
    Engine, // cur not used
    LoadDataBase,
    RemoveColumn,
    AddColumn,
    InvalidType,
    PrimaryKey,
    InterruptedRead,
    OutOfBounds,
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl  From<EncodingError> for Error {
    fn from(err: EncodingError) -> Error {
        Error::BinEn(err)
    }
}

impl From<DecodingError> for Error {
    fn from(err: DecodingError) -> Error {
        Error::BinDe(err)
    }
}

impl From<::byteorder::Error> for Error {
    fn from(err: ::byteorder::Error) -> Error {
        Error::Byteorder(err)
    }
}

//---------------------------------------------------------------
// Engine
//---------------------------------------------------------------

/// Storage Engine Interface.
///
/// A storage engine, like MyISAM and InnoDB, is responsible for reading and
/// writing data to disk, maintain and use indices, check for data corruption
/// and repair corrupt files.
///
/// Each table in a database may use a different storage engine.
pub trait Engine {
    /// writes the table.dat file
    fn create_table(&mut self) -> Result<(), Error>;
    /// returns the table
    fn table(&self) -> &Table;

    /// Writes a row to hard drive
    fn insert_row(&mut self, data: &[Option<ast::DataSrc>])
        -> Result<(), Error>;

    fn full_scan(&self) -> Result<Rows, Error>;
}

#[repr(u8)]
#[derive(Clone,Copy,Debug,RustcDecodable, RustcEncodable)]
enum EngineID {
    FlatFile = 1,
    InvertedIndex,
}


// # Some information for the `storage` working group:
//
// You work at the very bottom of the database: The thing that actually
// writes the data disk. Everything in this module is used only by the
// query execution module.
//
// The file layout may look like this:
// We have some `db_dir` where everything lives. In that directory, there are
// subdirectories for every database. In each of those subdirs is optionally
// a file `db.meta` that contains information about the database (such as
// permissions). The tables of each database are saved in *.tbl files that
// live inside the database directory.
//
// Your task is to provide types and method to:
// - read a specific table from a specific
//   database from file
// - create a new table in a database
// - query meta information about a table (columns for example)
// - lock a table for reading/writing it's data through a storage engine
//
// The other main task is to:
// - specify the storage engine interface
// - implement a simple storage engine (heap/flatfiles)
//
