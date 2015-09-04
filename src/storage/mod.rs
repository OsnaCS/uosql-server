//! Storage Engine trait and several implementations
//!
//!


pub mod flatfile;

use self::flatfile::FlatFile;

use super::parse::ast::SqlType;

use std::mem;

use std::io;
use std::io::prelude::*;

use std::fs;
use std::fs::{OpenOptions, create_dir};

use std::convert::From;

use byteorder::Error;
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};

use bincode::SizeLimit;
use bincode::rustc_serialize::{EncodingError,
    DecodingError, encode_into, decode_from};

/// constants
 const MAGIC_NUMBER: u64 = 0x6561742073686974; // secret
 const VERSION_NO: u8 = 1;


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
    RemoveColumn,
    AddColumn,
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

//---------------------------------------------------------------
// DataType
//---------------------------------------------------------------
/// A Enum for Datatypes (will be removed later)
#[repr(u8)]
#[derive(Clone,Copy,Debug,RustcDecodable, RustcEncodable)]
pub enum DataType { Integer = 1, Float = 2, }

impl DataType {
    pub fn value(&self) -> u8 {
       *self as u8
    }
}

//---------------------------------------------------------------
// Database
//---------------------------------------------------------------
#[derive(Debug)]
pub struct Database {
    name: String,
}

impl Database {
    /// Starts the process of creating a new Database
    /// Returns database or on fail DatabaseError
    pub fn create(name: &str) -> Result<Database, DatabaseError> {
        let d = Database{ name: name.to_string() };
        try!(d.save());
        info!("created new database {:?}",d);
        Ok(d)
    }

    /// Loads already existing Database
    /// returns DataBase Error when database does not exist else the loaded DB
    pub fn load(name: &str) -> Result<Database, DatabaseError> {
        if try!(fs::metadata(name)).is_dir() {
            info!("loaded Database {:?}", name.to_string());
            Ok(Database{ name: name.to_string() })
        } else {
            warn!("could not load database {:?}", name.to_string());
            return Err(DatabaseError::LoadDataBase)
        }
    }

    /// Creates a folder for the database
    fn save(&self) -> Result<(), DatabaseError> {
        info!("trying to create dir!");
        try!(create_dir(&self.name));
        info!("created dir");
        Ok(())
    }

    /// Creates a new table in the DB folder
    /// Returns with DatabaseError on fail else Table
    pub fn create_table(&self, name: &str, columns: Vec<Column>, engine_id: u8)
        -> Result<Table, DatabaseError>
    {

        let t = Table::new(&self, name, columns, engine_id);
        try!(t.save());
        info!("created new table {:?}", t);
        Ok(t)
    }

    /// calls load for table with the database path
    /// Returns with DatabaseError on fail else Table
    pub fn load_table(&self, name: &str) -> Result<Table, DatabaseError> {
        Table::load(&self, name)
    }
}


//---------------------------------------------------------------
// TableMetaData
//---------------------------------------------------------------

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct  TableMetaData {
    version_nmbr: u8,
    engine_id: u8,
    columns: Vec<Column>,
}

//---------------------------------------------------------------
// Table
//---------------------------------------------------------------

/// Table struct that contains the table information
#[derive(Debug)]
pub struct Table<'a> {
    database: &'a Database,
    name: String,
    meta_data: TableMetaData,
}

impl<'a> Table<'a> {
    /// Creates new table object
    /// Returns Table
    pub fn new<'b>(database: &'b Database, name: &str,
                   columns: Vec<Column>, engine_id: u8)
        -> Table<'b>
    {

        let meta_data = TableMetaData {
            version_nmbr: VERSION_NO,
            engine_id: engine_id,
            columns: columns,
        };
        info!("created meta data: {:?}", meta_data);

        Table {
            name: name.to_string(),
            database: database,
            meta_data: meta_data,
        }
    }

    /// Loads the table from the DB
    /// Returns with DatabaseError on fail else Table
    fn load<'b>(database: &'b Database, name: &str)
        -> Result<Table<'b>, DatabaseError>
    {
        // TODO: Read the .tbl file from disk and parse it

        let path_to_table = Table::get_path(&database.name, name, "tbl");
        info!("getting path and opening file: {:?}", path_to_table);
        let mut file = try!(OpenOptions::new()
            .read(true)
            .open(path_to_table));
        info!("reading file: {:?}", file);
        let ma_nmbr = try!(file.read_uint::<BigEndian>(mem::size_of_val(&MAGIC_NUMBER)));

        info!("checking magic number: {:?}",ma_nmbr);
        if ma_nmbr != MAGIC_NUMBER {
            println!("Magic Number not correct");
            return Err(DatabaseError::WrongMagicNmbr)
        }
        let meta_data: TableMetaData = try!(decode_from(&mut file, SizeLimit::Infinite));
        info!("getting meta data{:?}", meta_data);

        let table = Table::new(database, name, meta_data.columns, meta_data.engine_id);
        info!("returning table: {:?}", table);
        Ok(table)
    }

    /// Saves the table with a identification number in table file
    /// Returns DatabaseError on fail else Nothing
    pub fn save(&self) -> Result<(), DatabaseError> {
        // call for open file
        info!("opening file to write",);
        let mut file = try!(OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.get_table_metadata_path()));
        info!("writing magic number in file: {:?}", file);
        try!(file.write_u64::<BigEndian>(MAGIC_NUMBER));//MAGIC_NUMBER
        info!("writing meta data in file: {:?}",file);
        try!(encode_into(&self.meta_data, &mut file,SizeLimit::Infinite));

        // debug message all okay
        info!("I Wrote my File");
        Ok(())
    }

    /// Deletes the .tbl files
    /// Returns DatabaseError on fail if path points to a directory,
    /// if the user lacks permissions to remove the file,
    /// or if some other filesystem-level error occurs.
    pub fn delete(&self) -> Result<(), DatabaseError>{

        info!("remove meta file: {:?}", self.get_table_metadata_path());
        try!(fs::remove_file(self.get_table_metadata_path()));

        info!("remove data file: {:?}",self.get_table_data_path());
        try!(fs::remove_file(self.get_table_data_path()));

        Ok(())
    }

    /// Returns columns of table as array
    pub fn columns(&self) -> &[Column] {
        &self.meta_data.columns
    }

    /// Adds a column to the tabel
    /// Returns name of Column or on fail DatabaseError
    pub fn add_column(&mut self, name: &str, sql_type: SqlType) -> Result<(), DatabaseError> {
        match self.meta_data.columns.iter().find(|x| x.name == name) {
            Some(_) => {
                warn!("Column {:?} already exists", name);
                return Err(DatabaseError::AddColumn)
            },
            None => {
                info!("Column {:?} was added", name);
            },
        }
        self.meta_data.columns.push(Column::create_new(name, sql_type));
        Ok(())
    }

    /// Removes a column from the table
    /// Returns name of Column or on fail DatabaseError
    pub fn remove_column(&mut self, name: &str) -> Result<(), DatabaseError> {
        let index = match self.meta_data.columns.iter().position(|x| x.name == name) {
            Some(x) => {
                info!("Column {:?} was removed" , self.name);
                x
            },
            None => {
                warn!("Column {:?} could not be found", self.name);
                return Err(DatabaseError::RemoveColumn)
            },
        };
        self.meta_data.columns.swap_remove(index);
        Ok(())
    }

    /// Creates an engine for Table
    /// Returns Box<Engine>
    pub fn create_engine(self) -> Box<Engine + 'a> {
        Box::new(FlatFile::new(self))
    }

    /// Returns the path for the metadata files
    fn get_table_metadata_path(&self) -> String {
        Self::get_path(&self.database.name, &self.name, "tbl")
    }

    /// Returns the path for the data files
    fn get_table_data_path(&self) -> String {
        Self::get_path(&self.database.name, &self.name, "dat")
    }

    /// Returns the path of the table
    fn get_path(database: &str, name: &str, ext: &str) -> String {
         format!("{}/{}.{}", database, name, ext)
    }
}

//---------------------------------------------------------------
// Column
//---------------------------------------------------------------

/// A table column. Has a name, a type, ...
#[derive(Debug,RustcDecodable, RustcEncodable,Clone)]
pub struct Column {
    pub name: String, //name of column
    pub sql_type: SqlType, //name of the data type that is contained in this column
}


impl Column {
    /// Creates a new column object
    /// Returns with Column
    pub fn create_new(name: &str, sql_type: SqlType) -> Column {
        Column {
            name: name.to_string(),
            sql_type: sql_type.clone()
        }
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
    fn create_table(&mut self) -> Result<(), DatabaseError>;
}

#[repr(u8)]
#[derive(Clone,Copy,Debug,RustcDecodable, RustcEncodable)]
enum EngineID {
    FlatFile = 1,
    InvertedIndex = 2,
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
