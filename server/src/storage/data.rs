use std::vec::Vec;
use super::Error;
use super::types::SqlType;
use super::types::Column;
use std::io::{Write, Read};
use std::io::{BufReader, BufWriter};

#[derive(Debug)]
pub struct Rows <B: Write + Read> {
    buf_steam: BufStream,
    columns: Vec<Column>,
}

/// Represents the lines read from file.
impl<'a, B: Write + Read> Rows <'a, B> {

    pub fn new(data_src: B, columns: &[Column]) -> Rows<B> {
        Rows { buf: data_src,
               buf_reader: None,
               buf_writer: None,
               columns: columns.to_vec() }
    }

    /// reads the next row, which is not marked as deleted
    /// and writes it into target_buf
    /// returns true, if the next row could be read successfully
    pub fn next_row<W: Write>(&mut self, mut target_buf: &W) -> bool {
        false
    }

    /// checks if a next line exists
    fn has_next(&self) -> bool {
        false
    }

    /// reads the current row and writes the data into target_buf
    /// returns the bytes read
    pub fn read_row<W: Write>(&self, target_buf: &W) -> Result<u64, Error> {
        Err(Error::NoImplementation)
    }

    /// sets position before the first line
    pub fn reset_pos(&mut self) {
    }

    /// sets position to offset
    fn set_pos(&mut self, offset: u64) {

    }

    /// reads the header of the current row
    /// returns an error if no RowHeader exists
    pub fn read_header(&self) -> Result <RowHeader, Error> {
        Err(Error::NoImplementation)
    }

    /// writes a new row into buf, returns bytes written
    pub fn add_row(&self, row_data: &[u8]) -> Result<u64, Error> {
        Err(Error::NoImplementation)
    }

    /// returns the value of the column_index' column of the current row
    /// returns an error if no current row exists
    pub fn get_value(&self, column_index: usize) -> Result<Vec<u8>, Error> {
        Err(Error::NoImplementation)
    }

    /// returns true if a row can be read
    fn on_row(&self) -> bool {
        false
    }

    fn get_buf_reader(&mut self) -> &BufReader<B> {
        match self.buf_reader {
            None => {
                self.buf_reader = Some(&BufReader::new(self.buf));
                &self.buf_reader.unwrap()
            }
            Some(v) => { v }
        }
    }

}

pub struct RowHeader{
    data: u8,
}

impl RowHeader{
    /// returns true if the current row is marked as deleted
    pub fn is_deleted(&self) -> bool {
        false
    }

    /// marks row as deleted
    pub fn set_deleted(&mut self) {

    }
}
