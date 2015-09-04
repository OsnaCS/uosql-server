use super::{Engine, DatabaseError, Column};

use std::fs::OpenOptions;

pub struct FlatFile {
    table_path: String,
}

impl FlatFile {
    pub fn new(path_to_table: String) -> FlatFile {
        println!("Hallo");
        FlatFile { table_path: path_to_table }
    }
}

impl Drop for FlatFile {
    fn drop(&mut self) {
        println!("Tschüss");
    }
}

impl Engine for FlatFile {
    fn create_table(&mut self, _cols: &[Column]) -> Result<(), DatabaseError> {
        let mut _file = try!(OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.table_path));
        Ok(())
    }
}
