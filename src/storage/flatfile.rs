use super::{Engine, DatabaseError, Table};
use std::fs::OpenOptions;

pub struct FlatFile<'a> {
    table: Table<'a>,
}

impl<'a> FlatFile<'a> {
    pub fn new<'b>(table: Table<'b>) -> FlatFile<'b> {
        println!("Hallo");
        FlatFile { table: table }
    }
}

impl<'a> Drop for FlatFile<'a> {
    fn drop(&mut self) {
        println!("Tschüss");
    }
}

impl<'a> Engine for FlatFile<'a> {
    fn create_table(&mut self) -> Result<(), DatabaseError> {
        let mut _file = try!(OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.table.get_table_data_path()));
        Ok(())
    }
}
