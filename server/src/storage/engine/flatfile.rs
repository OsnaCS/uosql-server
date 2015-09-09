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

    /// Insert values from data into rows of the table
    fn insert_row(&mut self, data: &[Option<ast::DataSrc>])
                  -> Result<(), Error> {

        // Open table data file
        let mut file = try!(OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(&self.table.get_table_data_path()));
        info!("created file for data: {:?}", file);

        let defaults = [ast::DataSrc::Int(0),
                        ast::DataSrc::Bool(0),
                        ast::DataSrc::String("l".to_string()),
                        ast::DataSrc::String("o".to_string())];

        // Iterate over given columns data and the meta data
        // simultaneously and get either the given data or a
        // defaul type
        info!("starting encodeding of data");
        for (d, meta) in data.iter().zip(self.table.columns()) {
            // Entry contains default or given value
            let entry = d.as_ref().unwrap_or(match meta.sql_type {
                SqlType::Int => &defaults[0],
                SqlType::Bool => &defaults[1],
                SqlType::Char(_) => &defaults[2],
                SqlType::VarChar(_) => &defaults[3],
            });

            // Try to encode the data entry into the table file
            // (appends to end of file)
            try!(meta.sql_type.encode_into(&mut file, entry));
        }
        info!("finished encoding");
        Ok(())
    }

    fn full_scan(&self) -> Result<Rows, Error> {
        let bytes_to_read =
            try!(fs::metadata(&self.table.get_table_data_path())).len();

        let mut file = try!(OpenOptions::new()
                            .read(true)
                            .open(&self.table.get_table_data_path()));

        let mut buf = Vec::<u8>::new();

        let bytes_read = try!(file.read_to_end(&mut buf)) as u64;

        if bytes_to_read != bytes_read {
            return Err(Error::InterruptedRead);
        }

        Ok(Rows{data: buf, table: self.table()})
    }
}
