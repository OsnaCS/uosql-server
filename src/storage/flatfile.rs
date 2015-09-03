use super::{Engine, DatabaseError, Column};

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
    fn create_table(&mut self, cols: &[Column]) -> Result<(), DatabaseError> {
        Ok(())
    }
}
