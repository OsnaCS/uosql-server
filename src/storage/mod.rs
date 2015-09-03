//! Storage Engine trait and several implementations
//!
//!


use std::path::Path;
use std::convert::From;
use byteorder::Error;
use std::io::prelude::*;
use std::io;
use std::fs::{ OpenOptions,File,create_dir };

use byteorder::{ WriteBytesExt, ReadBytesExt, BigEndian };

use bincode::SizeLimit;
use bincode::rustc_serialize::{ EncodingError,DecodingError,encode_into,decode_from };

/// constants
 const MAGIC_NUMBER: u64 = 0x6561742073686974; // secret
 const MAGIC_NUMBER_BYTES: usize = 8;


/// Storage Engine Interface.
///
/// A storage engine, like MyISAM and InnoDB, is responsible for reading and
/// writing data to disk, maintain and use indices, check for data corruption
/// and repair corrupt files.
///
/// Each table in a database may use a different storage engine.
pub trait Engine {

}

/// A database table
///
/// Through this type, you can retreive certain meta information about the
/// table (like column names, column types, storage engine, ...). It's `access`
/// method locks the table globally and returns a storage engine to access
/// the table data.
#[derive(Debug)]
pub enum DatabaseError {
    Io(io::Error),
    BinEn(EncodingError),
    BinDe(DecodingError),
    Byteorder(::byteorder::Error),
    MagicNmbr,
}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> DatabaseError {
        DatabaseError::Io(err)
    }
}

impl  From<EncodingError> for DatabaseError {
    fn from(err: EncodingError) -> DatabaseError {
        DatabaseError::BinEn(err)
    }
}

impl From<DecodingError> for DatabaseError {
    fn from(err: DecodingError) -> DatabaseError {
        DatabaseError::BinDe(err)
    }
}

impl From< ::byteorder::Error> for DatabaseError {
    fn from(err: ::byteorder::Error) -> DatabaseError {
        DatabaseError::Byteorder(err)
    }
}

/// A Enum for File Modes
///
/// Can be used to define the save and load configuration of open_file
#[derive(Debug)]
enum FileMode{ LoadDefault, SaveDefault, }

/// A Enum for Datatypes (will be removed later)
#[repr(u8)]
#[derive(Clone,Copy,Debug,RustcDecodable, RustcEncodable)]
pub enum DataType{ Integer = 1, Float = 2,}
impl DataType {
    pub fn value(&self) -> u8 {
       *self as u8
    }
}

pub struct Database {
    name: String,
}

impl Database {
    pub fn new_database(database_name: &str) -> Database {
        Database{ name: database_name.to_string() }
    }
    pub fn create_database(&self) -> Result<(),DatabaseError> {
        println!("trying to create dir!");
        try!(create_dir(&self.name));
        println!("created dir");
        Ok(())
    }
}

#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct Table {
    name: String,
    engine_id: u8,
    version_nmbr: u8,
    column_nmbr: u16,
    columns: Vec<Column>,

}

impl Default for Table {
    fn default() -> Table {
        let mut t: Vec<Column> = Vec::new();
        t.push(Default::default());
        Table { name:"default".to_string(), engine_id: 3, version_nmbr:  1, column_nmbr: 1, columns: t }
    }
}

impl Table {
    pub fn load(database: &str, table: &str) -> Result<(), DatabaseError> {
        // TODO: Read the .tbl file from disk and parse it
        let mut file = try!(Self::open_file(database,table,FileMode::LoadDefault));

        let ma_nmbr = try!(file.read_uint::<BigEndian>(MAGIC_NUMBER_BYTES));

        if ma_nmbr != MAGIC_NUMBER {
            println!("Magic Number not correct");
            return Err(DatabaseError::MagicNmbr)
        }
        let data: Table = try!(decode_from(&mut file, SizeLimit::Infinite));
        println!("{:?}", data);
        Ok(())
    }

    pub fn create_new(engine: u8, name: &str) -> Table {

        Table { name: name.to_string(), engine_id: engine, version_nmbr: 1, column_nmbr: 0, columns: Vec::new() }
    }

    pub fn save(&self,database: &str, table: &str) -> Result<(), DatabaseError> {
        // call for open file
        let mut file = try!(Self::open_file(database,table,FileMode::SaveDefault));
        try!(file.write_u64::<BigEndian>(MAGIC_NUMBER));//MAGIC_NUMBER
        try!(encode_into(&self,&mut file,SizeLimit::Infinite));

        // debug message all okay
        println!("I Wrote my File");
        Ok(())
    }

    fn open_file(database: &str, table: &str, mode: FileMode) -> Result<File, DatabaseError> {
        // create new file or open new one
        let st = format!("{}/{}",database,table);
        let path = Path::new(&st);


        match mode {
            FileMode::SaveDefault => OpenOptions::new()
                .write(true)
                .create(true)
                .open(path),
            FileMode::LoadDefault => OpenOptions::new()
                .read(true)
                .open(path),
        }.map_err(|e| e.into())
    }

    pub fn columns(&self) -> &[Column] {
        // TODO: Return real columns
        &self.columns
    }

    pub fn add_column(&mut self, name: &str, dtype: DataType) {
        self.columns.push(Column::create_new(name, dtype));
    }

    pub fn remove_column(&mut self, name: &str, data_type: DataType) {

    }
}

/// A table column. Has a name, a type, ...
#[derive(Debug,RustcDecodable, RustcEncodable,Clone)]
pub struct Column {
    name: String, //name of column
    data_type: DataType, //name of the data type that is contained in this column
}

impl Default for Column {
    fn default() -> Column
    {
        Column{name: "default".to_string(), data_type: DataType::Integer}
    }
}

impl Column {
    pub fn create_new(name: &str, dtype: DataType) -> Column {
        Column{ name: name.to_string(), data_type: dtype.clone() }
    }
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
