use super::super::meta::{Table};
use super::super::{Engine, Error};
use std::fs::OpenOptions;
use std::io::Read;
use super::super::super::parse::ast;
use super::super::types::SqlType;

//---------------------------------------------------------------
// FlatFile-Engine
//---------------------------------------------------------------
use super::super::data::{Rows};
use std::fs;

pub struct FlatFile<'a> {
    table: Table<'a>,
}

impl<'a> FlatFile<'a> {
    ///
    pub fn new<'b>(table: Table<'b>) -> FlatFile<'b> {
        info!("new flatfile with table: {:?}", table);
        FlatFile { table: table }
    }
}

impl<'a> Drop for FlatFile<'a> {
    /// drops the Flatfile
    fn drop(&mut self) {
        info!("drop engine flatfile");
    }
}

impl<'a> Engine for FlatFile<'a> {
    /// creates table for use later
    /// returns with error when it has either no permission or full disk
    fn create_table(&mut self) -> Result<(), Error> {
        let mut _file = try!(OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.table.get_table_data_path()));

        info!("created file for data: {:?}", _file);

        Ok(())
    }
    /// returns own table
    fn table(&self) -> &Table {
        &self.table
    }

}
