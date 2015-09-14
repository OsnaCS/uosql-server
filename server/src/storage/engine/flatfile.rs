use super::super::meta::{Table};
use super::super::{Engine, Error};
use std::fs::{OpenOptions, File};
use std::io::{Write, Read, Seek, SeekFrom, Cursor};
use super::super::super::parse::ast;
use super::super::super::parse::ast::CompType;
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

    pub fn get_reader(&self) -> Result<Rows<File>, Error> {
        let mut file = try!(OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.table.get_table_data_path()));
        info!("opened file {:?}", file);

        Ok(Rows::new(file, &self.table.meta_data.columns))
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


    fn full_scan(&self) -> Result<Rows<Cursor<Vec<u8>>>, Error> {
        let mut reader = try!(self.get_reader());
        let vec: Vec<u8> = Vec::new();
        let cursor = Cursor::new(vec);
        let mut rows = Rows::new(cursor, &self.table.meta_data.columns);
        let mut buf: Vec<u8> = Vec::new();

        while true {
            match reader.next_row(&mut buf) {
                Ok(_) => {
                        rows.add_row(& buf);
                },
                Err(e) => {
                    match e {
                        Error::EndOfFile => break,
                        _ => return Err(e)
                    }
                },
            }
        }
        Ok(rows)
    }

    fn lookup(&self, column_index: usize, value: &[u8], comp: CompType)
    -> Result<Rows<Cursor<Vec<u8>>>, Error>
    {
        let mut reader = try!(self.get_reader());
        let vec: Vec<u8> = Vec::new();
        let cursor = Cursor::new(vec);
        let mut rows = Rows::new(cursor, &self.table.meta_data.columns);
        let mut buf: Vec<u8> = Vec::new();

        while true {
            match reader.next_row(&mut buf) {
                Ok(_) => {
                    let col = reader.get_column(column_index);
                    if try!(col.sql_type.cmp(&try!(reader.get_value(&buf, column_index)), value, comp)) {
                        rows.add_row(& buf);
                    }
                },
                Err(e) => {
                    match e {
                        Error::EndOfFile => break,
                        _ => return Err(e)
                    }
                },
            }
        }

        Ok(rows)
    }

}
