//! Storage Engine trait and several implementations
//!
//!


pub mod flatfile;

use self::flatfile::FlatFile;
use std::fs::{self};
use std::convert::From;
use byteorder::Error;
use std::io::prelude::*;
use std::io;
use std::fs::{OpenOptions,create_dir};
use std::mem;

use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};

use bincode::SizeLimit;
use bincode::rustc_serialize::{EncodingError,DecodingError,encode_into,decode_from};

/// constants
 const MAGIC_NUMBER: u64 = 0x6561742073686974; // secret
 const VERSION_NO: u8 = 1;


/// Storage Engine Interface.
///
/// A storage engine, like MyISAM and InnoDB, is responsible for reading and
/// writing data to disk, maintain and use indices, check for data corruption
/// and repair corrupt files.
///
/// Each table in a database may use a different storage engine.
pub trait Engine {
    fn create_table(&mut self, cols: &[Column]) -> Result<(), DatabaseError>;
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
    WrongMagicNmbr,
    Engine, //cur not used
    LoadDataBase,
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
enum _FileMode{ LoadDefault, SaveDefault, }

/// A Enum for Datatypes (will be removed later)
#[repr(u8)]
#[derive(Clone,Copy,Debug,RustcDecodable, RustcEncodable)]
pub enum DataType{ Integer = 1, Float = 2, }

impl DataType {
    pub fn value(&self) -> u8 {
       *self as u8
    }
}

pub struct Database {
    name: String,
}

impl Database {
    /// Starts the process of creating a new Database
    /// Returns database or on fail DatabaseError
    pub fn new_database(database_name: &str) -> Result<Database, DatabaseError> {
        let d = Database{ name: database_name.to_string() };
        try!(d.create_database());
        Ok(d)
    }

    /// Loads already existing Database
    /// returns DataBase Error when database does not exist else the loaded DB
    pub fn load_database(database_name: &str) -> Result<Database, DatabaseError> {
        if try!(fs::metadata(database_name)).is_dir() {
            Ok(Database{ name: database_name.to_string() })
        } else {
            return Err(DatabaseError::LoadDataBase)
        }
    }

    /// Creates a folder for the database
    fn create_database(&self) -> Result<(), DatabaseError> {
        println!("trying to create dir!");
        try!(create_dir(&self.name));
        println!("created dir");
        Ok(())
    }

    /// Creates a new table in the DB folder
    /// Returns with DatabaseError on fail else Table
    pub fn create_table(&self, engine_id: u8, cols: Vec<Column>, table_name: &str)
        -> Result<Table, DatabaseError> {

        let t = Table::new(engine_id, &self.name, table_name, cols);
        try!(t.save());
        Ok(t)
    }

    /// calls load for table with the database path
    /// Returns with DatabaseError on fail else Table
    pub fn load_table(&self, table_name: &str) -> Result<Table, DatabaseError> {
        Self::load(&self.name, table_name)
    }

    /// Loads the table from the DB
    /// Returns with DatabaseError on fail else Table
    fn load(database: &str, table: &str) -> Result<Table, DatabaseError> {
        // TODO: Read the .tbl file from disk and parse it
        let path_to_table = Table::get_path(database, table, "tbl");
        let mut file = try!(OpenOptions::new()
            .read(true)
            .open(path_to_table));

        let ma_nmbr = try!(file.read_uint::<BigEndian>( mem::size_of_val(&MAGIC_NUMBER)));

        if ma_nmbr != MAGIC_NUMBER {
            println!("Magic Number not correct");
            return Err(DatabaseError::WrongMagicNmbr)
        }
        let table: Table = try!(decode_from(&mut file, SizeLimit::Infinite));
        println!("{:?}", table);

        Ok(table)
    }
}

/// Table struct that contains the table information
#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Table {
    version_nmbr: u8,
    engine_id: u8,
    columns: Vec<Column>,
    name: String,
    name_of_database: String,
}



impl Table {
    /// Creates new table object
    /// Returns Table
    pub fn new(engine: u8, database: &str, table_name: &str, cols: Vec<Column>) -> Table {
        Table {
            version_nmbr: VERSION_NO,
            engine_id: engine,
            columns: cols,
            name: table_name.to_string(),
            name_of_database: database.to_string(),
        }
    }

    /// Saves the table with a identification number in table file
    /// Returns DatabaseError on fail else Nothing
    pub fn save(&self) -> Result<(), DatabaseError> {
        // call for open file
        let mut file = try!(OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.get_table_metadata_path()));

        try!(file.write_u64::<BigEndian>(MAGIC_NUMBER));//MAGIC_NUMBER
        try!(encode_into(&self, &mut file,SizeLimit::Infinite));

        // debug message all okay
        println!("I Wrote my File");
        Ok(())
    }

    /// Returns columns of table as array
    pub fn columns(&self) -> &[Column] {
        // TODO: Return real columns
        &self.columns
    }

    /// Adds a column to the tabel
    pub fn add_column(&mut self, name: &str, dtype: DataType) {
        self.columns.push(Column::create_new(name, dtype));
    }

    /// Removes a column from the table
    pub fn remove_column(&mut self, _name: &str, _data_type: DataType) {
    }

    /// Creates an engine for Table
    /// Returns Box<Engine>
    pub fn create_engine(&self) -> Box<Engine> {
        Box::new(FlatFile::new(self.get_table_data_path()))
    }

    /// Returns the path for the metadata files
    fn get_table_metadata_path(&self) -> String {
        Self::get_path(&self.name_of_database, &self.name, "tbl")
    }

    /// Returns the path for the data files
    fn get_table_data_path(&self) -> String {
        Self::get_path(&self.name_of_database, &self.name, "dat")
    }

    /// Returns the path of the table
    fn get_path(database: &str, name: &str, ext: &str) -> String {
         format!("{}/{}.{}", database, name, ext)
    }
}

/// A table column. Has a name, a type, ...
#[derive(Debug,RustcDecodable, RustcEncodable,Clone)]
pub struct Column {
    pub name: String, //name of column
    pub data_type: DataType, //name of the data type that is contained in this column
}

impl Default for Column {
    /// Returns a default Column construct
    fn default() -> Column {
        Column {
            name: "default".to_string(),
            data_type: DataType::Integer
        }
    }
}

impl Column {
    /// Creates a new column object
    /// Returns with Column
    pub fn create_new(name: &str, dtype: DataType) -> Column {
        Column {
            name: name.to_string(),
            data_type: dtype.clone()
        }
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
