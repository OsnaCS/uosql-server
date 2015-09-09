use std::vec::Vec;
use super::Table;
use super::Error;
use super::super::parse::ast::DataSrc;
use super::types::SqlType;
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
pub struct Rows<'a> {
    pub data: Vec<u8>,
    pub table: &'a Table<'a>,
}

pub struct RowsIter<'a> {
    rows : &'a Rows<'a>,
    iter_pos: u32,
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
    type Item = Vec<DataSrc>;

    fn next(&mut self) -> Option<Vec<DataSrc>> {

        if self.iter_pos >= self.rows.data.len() as u32 {
            return None;
        }

        let columns = self.rows.table.columns();
        let mut result = Vec::<DataSrc>::new();

        for i in 0..columns.len() {
            let mut col_data = match columns[i].get_sql_type() {
                &SqlType::VarChar(_) => {

                    let mut buf = &self.rows.data[(self.iter_pos as usize)
                            ..(self.iter_pos + 2) as usize];

                    let len = match buf.read_u16::<BigEndian>() {
                        Ok(len) => len,
                        Err(e) => return None
                    };

                    self.iter_pos = self.iter_pos + 2;

                    buf = &self.rows.data[(self.iter_pos as usize)
                        ..((self.iter_pos + len as u32) as usize)];
                    self.iter_pos = self.iter_pos + len as u32;
                    buf
                },
                _ => {
                    let mut buf =
                        &self.rows.data[(self.iter_pos as usize)
                        ..((self.iter_pos + columns[i].get_size()) as usize)];
                    self.iter_pos = self.iter_pos + columns[i].get_size();
                    buf
                }
            };

            let datasrc = match columns[i].sql_type.decode_from(&mut col_data) {
                Ok(d) => d,
                Err(e) => return None
            };
            result.push(datasrc);
        }

        Some(result)
    }
}
