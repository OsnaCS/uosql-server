use std::vec::Vec;
use super::Table;
use super::Error;
use super::super::parse::ast::DataSrc;
use super::types::SqlType;
use super::types::Column;
use super::types::FromSql;
use byteorder::{BigEndian, ReadBytesExt};

/// Contains binary data
#[derive(Debug)]
pub struct Rows {
    pub data: Vec<u8>,
    pub columns: Vec<Column>,
}

/// Represents a single row in a Rows struct
pub struct Row<'a> {
    owner: &'a Rows,
    // Contains one vector per column. Each vector contains
    // a vector with the column data.
    pub column_data: Vec<Vec<u8>>,
}

impl<'a> Row<'a> {
    /// Creates a new Row
    /// rows: the data struct containing the column definitions
    /// row_data: The binary data representing the row.
    pub fn new(rows: &'a Rows, row_data: &[u8]) -> Row<'a> {
        let mut row = Row {owner: rows, column_data: Vec::<Vec<u8>>::new()};
        row.load_data(row_data);
        row
    }

    /// returns the number of columns of the row.
    fn column_count(&self) -> u32 {
        self.owner.columns.len() as u32
    }

    //fn get_value_by_name<T: FromSql>(&self, col_name: &str) -> T {

    //}
    /// Gets the value of column index converted to type T
    /// Returns an error, if column type is not compatible with T
    pub fn get_value<T: FromSql>(&self, index: usize) -> Result<T, Error> {
        let v = &self.column_data[index];
        T::from_sql(v)
    }

    fn load_data(&mut self, row_data: &[u8]) -> Result<(), Error> {
        let columns = &self.owner.columns;
        let mut pos: usize = 0;

        for i in 0..columns.len() {
            let mut col_data = match columns[i].get_sql_type() {
                &SqlType::VarChar(_) => {

                    let raw_varchar_len = &row_data[pos..pos + 2];
                    let varchar_len = try!(u16::from_sql(raw_varchar_len));

                    pos = pos + 2;

                    let buf = &row_data[pos..pos + varchar_len as usize];
                    pos = pos + varchar_len as usize;
                    buf
                },
                _ => {
                    let mut buf =
                        &row_data[pos..pos + columns[i].get_size() as usize];
                    pos = pos + columns[i].get_size() as usize;
                    buf
                }
            };


            let mut v = Vec::<u8>::new();
            v.extend(col_data.iter().cloned());
            self.column_data.push(v);
        }
        Ok(())
    }
}



pub struct RowsIter<'a> {
    rows: &'a Rows,
    iter_pos: usize,
}

/// Represents the lines read from file.
impl Rows {
    /// Returns an iterator
    pub fn iter(&self) -> RowsIter {
        RowsIter {
            rows: self,
            iter_pos: 0
        }
    }
}

/// Implementation of Iterator
impl<'a> Iterator for RowsIter<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Row<'a>> {
        if self.iter_pos >= self.rows.data.len() {
            return None;
        }

        let columns = &self.rows.columns;
        let start_of_row = self.iter_pos;

        for i in 0..columns.len() {
            match columns[i].get_sql_type() {
                &SqlType::VarChar(_) => {

                    let raw_varchar_len = &self.rows.data[self.iter_pos..self.iter_pos + 2];

                    let varchar_len = match u16::from_sql(raw_varchar_len) {
                        Ok(u) => u,
                        Err(e) => return None,
                    };

                    self.iter_pos = self.iter_pos + varchar_len as usize + 2;
                },
                _ => {
                    self.iter_pos = self.iter_pos + columns[i].get_size() as usize;
                }
            };
        };

        Some(Row::new(self.rows, &self.rows.data[start_of_row..self.iter_pos]))
    }
}
