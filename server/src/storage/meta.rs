use std::mem;

use std::io::prelude::*;

use std::fs;
use std::fs::{OpenOptions, create_dir, remove_dir_all};

use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode_into, decode_from};

use super::SqlType;

use super::Engine;
use super::Error;
use super::engine::FlatFile;
use super::types::Column;

/// constants
const MAGIC_NUMBER: u64 = 0x6561742073686974; // secret
const VERSION_NO: u8 = 1;




//---------------------------------------------------------------
// DataType
//---------------------------------------------------------------
/// A Enum for Datatypes (will be removed later)
#[repr(u8)]
#[derive(Clone, Copy, Debug, RustcDecodable, RustcEncodable)]
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
    pub name: String,
}

impl Database {
    /// Starts the process of creating a new Database
    /// Returns database or on fail Error
    pub fn create(name: &str) -> Result<Database, Error> {
        let d = Database{ name: name.to_string() };
        try!(d.save());
        info!("created new database {:?}", d);
        Ok(d)
    }

    /// Loads already existing Database
    /// returns DataBase Error when database does not exist else the loaded DB
    pub fn load(name: &str) -> Result<Database, Error> {
        if try!(fs::metadata(name)).is_dir() {
            info!("loaded Database {:?}", name.to_string());
            Ok(Database{ name: name.to_string() })
        } else {
            warn!("could not load database {:?}", name.to_string());
            return Err(Error::LoadDataBase)
        }
    }

    /// Creates a folder for the database
    fn save(&self) -> Result<(), Error> {
        info!("trying to create dir!");
        try!(create_dir(&self.name));
        info!("created dir");
        Ok(())
    }
    /// Deletes the database folder and all its contents
    /// do not use RANDOM!!
    pub fn delete(&self) -> Result<(), Error> {
        info!("deleting Database and all its tables");
        try!(remove_dir_all(&self.name));
        Ok(())
    }
    /// Creates a new table in the DB folder
    /// Returns with Error on fail else Table
    pub fn create_table(&self, name: &str, columns: Vec<Column>, engine_id: u8)
        -> Result<Table, Error>
    {

        let t = Table::new(&self, name, columns, engine_id);
        try!(t.save());
        info!("created new table {:?}", t);
        Ok(t)
    }

    /// calls load for table with the database path
    /// Returns with Error on fail else Table
    pub fn load_table(&self, name: &str) -> Result<Table, Error> {
        Table::load(&self, name)
    }
}


//---------------------------------------------------------------
// TableMetaData
//---------------------------------------------------------------

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct TableMetaData {
    version_nmbr: u8,
    engine_id: u8,
    columns: Vec<Column>,
    //primary_key: String
}

//---------------------------------------------------------------
// Table
//---------------------------------------------------------------

/// Table struct that contains the table information
#[derive(Debug)]
pub struct Table<'a> {
    database: &'a Database,
    pub name: String,
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
    /// Returns with Error on fail else Table
    fn load<'b>(database: &'b Database, name: &str)
        -> Result<Table<'b>, Error>
    {
        // TODO: Read the .tbl file from disk and parse it

        let path_to_table = Table::get_path(&database.name, name, "tbl");
        info!("getting path and opening file: {:?}", path_to_table);
        let mut file = try!(OpenOptions::new()
            .read(true)
            .open(path_to_table));
        info!("reading file: {:?}", file);
        let ma_nmbr = try!(file.read_uint::<BigEndian>(mem::size_of_val(&MAGIC_NUMBER)));

        info!("checking magic number: {:?}", ma_nmbr);
        if ma_nmbr != MAGIC_NUMBER {
            println!("Magic Number not correct");
            return Err(Error::WrongMagicNmbr)
        }
        let meta_data: TableMetaData = try!(decode_from(&mut file, SizeLimit::Infinite));
        info!("getting meta data{:?}", meta_data);

        let table = Table::new(database, name, meta_data.columns, meta_data.engine_id);
        info!("returning table: {:?}", table);
        Ok(table)
    }

    /// Saves the table with a identification number in table file
    /// Returns Error on fail else Nothing
    pub fn save(&self) -> Result<(), Error> {
        // call for open file
        info!("opening file to write");
        let mut file = try!(OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.get_table_metadata_path()));
        info!("writing magic number in file: {:?}", file);
        try!(file.write_u64::<BigEndian>(MAGIC_NUMBER));//MAGIC_NUMBER
        info!("writing meta data in file: {:?}", file);
        try!(encode_into(&self.meta_data, &mut file, SizeLimit::Infinite));

        // debug message all okay
        info!("I Wrote my File");
        Ok(())
    }

    /// Deletes the .tbl files
    /// Returns Error on fail if path points to a directory,
    /// if the user lacks permissions to remove the file,
    /// or if some other filesystem-level error occurs.
    pub fn delete(&self) -> Result<(), Error> {

        info!("remove meta file: {:?}", self.get_table_metadata_path());
        try!(fs::remove_file(self.get_table_metadata_path()));

        info!("remove data file: {:?}", self.get_table_data_path());
        try!(fs::remove_file(self.get_table_data_path()));

        Ok(())
    }

    /// Returns columns of table as array
    pub fn columns(&self) -> &[Column] {
        &self.meta_data.columns
    }

    /// Adds a column to the tabel
    /// Returns name of Column or on fail Error
    pub fn add_column(
        &mut self,
        name: &str,
        sql_type: SqlType,
        allow_null: bool,
        description: &str,
        is_primary_key: bool
        ) -> Result<(), Error> {

        match self.meta_data.columns.iter().find(|x| x.name == name) {
            Some(_) => {
                warn!("Column {:?} already exists", name);
                return Err(Error::AddColumn)
            },
            None => {
                info!("Column {:?} was added", name);
            },
        }

        self.meta_data.columns.push(Column::new(
            name,
            sql_type,
            allow_null,
            description,
            is_primary_key)
        );
        Ok(())
    }

    /// Removes a column from the table
    /// Returns name of Column or on fail Error
    pub fn remove_column(&mut self, name: &str) -> Result<(), Error> {
        let index = match self.meta_data.columns.iter().position(|x| x.name == name) {
            Some(x) => {
                info!("Column {:?} was removed", self.name);
                x
            },
            None => {
                warn!("Column {:?} could not be found", self.name);
                return Err(Error::RemoveColumn)
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
    pub fn get_table_data_path(&self) -> String {
        Self::get_path(&self.database.name, &self.name, "dat")
    }

    /// Returns the path of the table
    fn get_path(database: &str, name: &str, ext: &str) -> String {
         format!("{}/{}.{}", database, name, ext)
    }

 /*   fn get_primary_key(&self) -> Column {
        match self.meta_data.columns.iter().find(|x| x.name == self.meta_data.primary_key) {
            Some(x) => {
                info!("Primaty Key: {:?} was found", self.meta_data.primary_key);
                Ok(x)
            },
            None => {
                warn!("Primary Key: {:?} was not found", self.meta_data.primary_key);
                Err(Error::PrimaryKey)
            },
        }
    }*/
}
