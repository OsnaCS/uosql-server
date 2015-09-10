use std::vec::Vec;
use super::Error;
use super::types::SqlType;
use super::types::Column;
use std::io::{Write, Read, Seek};

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Rows <B: Write + Read + Seek> {
    buf: B,
    columns: Vec<Column>,
    pos: u64,
}

/// Represents the lines read from file.
impl<B: Write + Read + Seek> Rows <B> {

    /// sets pos to the beginning of the next non-deleted line
    /// returns false if no next line could be found
    pub fn next_row(&mut self) -> bool {
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
        self.set_pos(OUT_OF_RANGE);
    }

    /// sets position to offset
    fn set_pos(&mut self, offset: u64) {
        self.pos = offset;
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
