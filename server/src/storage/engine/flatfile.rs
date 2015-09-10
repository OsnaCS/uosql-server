use super::super::meta::{Table};
use super::super::{Engine, Error};
use std::fs::OpenOptions;
use std::io::Read;
use super::super::super::parse::ast;
use super::super::types::SqlType;
use super::super::data::Row;
use super::super::super::parse::ast::DataSrc;

//---------------------------------------------------------------
// FlatFile-Engine
//---------------------------------------------------------------
use super::super::data::{Rows};
use std::fs;

pub struct PrimaryKeyMap {
    pub column_name: String,
    pub primary_key_value: DataSrc,
}

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

        Ok(Rows{data: buf, columns: self.table().columns().to_vec()})
    }

    /// Searches rows with machtching primary keys
    /// returns vector of rows containing matching rows on fail returns early with Error
    /// returns an empty vector when no matches found
    fn get_row_with_primary_key(&self, primary_keys: Vec<PrimaryKeyMap>)
        -> Result<Rows, Error>
        {

        try!(self.check_for_primary_key(&primary_keys));

        let rows = try!(self.full_scan()); // full list of rows
        let it = rows.iter(); // iterator for row list
        let mut checked_rows = Rows::default(); // result list
        checked_rows.columns = self.table.meta_data.columns.clone();
        info!("start match search");
        'outer: for i in it { // outer iterator for rows
            'inner: for pk in &primary_keys { // inner iterator for keys
                match &pk.primary_key_value {
                    &DataSrc::Int(x) => {
                        let val: i32 = try!(i.get_value_by_name::<i32>(
                                &(pk.column_name.to_string())
                            ));// get value of row with column

                        if !val == (x as i32) {
                            continue 'outer // stop checking if one false
                        }
                    }
                    &DataSrc::Bool(x) => {
                        let val: bool = try!(i.get_value_by_name::<bool>(
                                &(pk.column_name.to_string())
                            ));

                        if !(val && DataSrc::to_bool(x)) {
                            continue 'outer
                        }
                    }
                    &DataSrc::String(ref x)=> {
                        let val: String = try!(i.get_value_by_name::<String>(
                                &(pk.column_name.to_string())
                            ));

                        if !(val != *x) {
                            continue 'outer
                        }
                    }
                }
            }
            checked_rows.add_row(i); // push matches to result list
        }
        info!("finished search for match");
        Ok(checked_rows) // return result list
    }

    fn check_for_primary_key(&self, primary_keys: &Vec<PrimaryKeyMap>) -> Result<bool, Error> {
        info!("start check for pk");
        for pk in primary_keys {
            match self.table.meta_data.columns.iter().find(|x| x.name == pk.column_name) {
                Some(x) => {
                    if x.is_primary_key == false {
                        warn!("Column {:?} is not a primary key", pk.column_name);
                        return Err(Error::NotAPrimaryKey)
                    }
                },
                None => {
                    warn!("Column {:?} does not exist", pk.column_name);
                    return Err(Error::InvalidColumn)
                },
            }
        }
        info!("check for pk was successful");
        Ok(true)
    }
}
