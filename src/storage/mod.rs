//! Storage Engine trait and several implementations
//!
//!


use byteorder::{BigEndian,ReadBytesExt,WriteBytesExt};
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::fs::File;
use std::io;
use bincode::rustc_serialize::{encode, decode,encode_into,decode_from};
use bincode::SizeLimit;

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

/// A Enum for File Modes
///
/// Can be used to define the save and load configuration of open_file
#[derive(Debug)]
enum FileMode{LoadDefault, SaveDefault,}

/// A Enum for Datatypes (will be removed later)
#[repr(u8)]
#[derive(Clone,Copy,Debug,RustcDecodable, RustcEncodable)]
enum DataType{ Integer = 1, Float = 2,}
impl DataType{
    pub fn value(&self) -> u8{
       *self as u8
    }
}
#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct Table {
    engine_id: u8,
    version_nmbr: u8,
    magic_nmbr: u8,
    column_nmbr: u16,
    columns: Vec<Column>,

}

impl Table {
    pub fn load(database: &str, table: &str) -> Result<(), io::Error> {
        // TODO: Read the .tbl file from disk and parse it
        let mut file = try!(Self::open_file(database,table,FileMode::LoadDefault));
        let data: Table = decode_from(&mut file, SizeLimit::Infinite).unwrap();
        println!("{:?}", data);
        Ok(())
    }

    pub fn create_new() -> Table {
        let mut t: Vec<Column> = Vec::new();
        t.push(Column::create_new());
        Table { engine_id: 3 , version_nmbr:  1, magic_nmbr: 170, column_nmbr: 1, columns: t}
    }

    pub fn save(&self,database: &str, table: &str) -> Result<(), io::Error> {
        //call for open file
        let mut file = try!(Self::open_file(database,table,FileMode::SaveDefault));

        let v = encode_into(&self,&mut file,SizeLimit::Infinite).unwrap();

        //debug message all okay
        println!("I Wrote my File");
        Ok(())
    }

    fn open_file(database: &str, table: &str, mode: FileMode) -> Result<(File), io::Error> {
        //create new file or open new one
        match mode {
            FileMode::SaveDefault => OpenOptions::new()
                .write(true)
                .create(true)
                .open(table),
            FileMode::LoadDefault => OpenOptions::new()
                .read(true)
                .open(table),
            }
    }

    pub fn columns(&self) -> &[Column] {
        // TODO: Return real columns
        &[]
    }
}

/// A table column. Has a name, a type, ...
#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct Column{
    name: String, //name of column
    data_type: DataType, //name of the data type that is contained in this column
}

impl Column{
    ///deprecated
    pub fn save_to_file(&self, file: &mut File) -> Result<(), io::Error> {
        //write column information down
        try!(file.write_u8(self.data_type.value()));
        try!(file.write_u8(self.name.len() as u8));
        try!(file.write_all(self.name.as_bytes()));

        Ok(())
    }

    pub fn create_new() -> Column{
        Column{name: "duh".to_string(), data_type: DataType::Integer}
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
