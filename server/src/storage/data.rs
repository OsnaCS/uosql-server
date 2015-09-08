use std::vec::Vec;
use super::Table;
use super::Error;
use super::super::parse::ast::DataSrc;

#[derive(Debug)]
pub struct Rows<'a> {
    pub data: Vec<u8>,
    pub table: &'a Table<'a>,
}

/// Represents the lines read from file.
impl<'a> Rows<'a> {
    /// Get size of row.
    fn get_row_size (&self) -> u64 {

        let columns = self.table.columns();

        let mut size = 0;

        for i in 0..columns.len() {
            size += columns[i].get_size();
        }

        size as u64
    }


    /// Gets index' row in data.
    pub fn get_row(&self, index : u64) -> Result<Vec<DataSrc>, Error> {

        let row_size = self.get_row_size();
        let mut result = Vec::<DataSrc>::new();

        let mut field_start = index * row_size;

        if field_start >= (self.data.len() as u64) {
            return Err(Error::OutOfBounds);
        }

        let columns = self.table.columns();

        for i in 0..columns.len() {
            let mut col_data = &self.data[(field_start as usize)..((field_start + columns[i].get_size()) as usize)];
            field_start = field_start + columns[i].get_size();
            let datasrc = try!(columns[i].sql_type.decode_from(&mut col_data));
            result.push(datasrc);
        }

        Ok(result)
    }
}
