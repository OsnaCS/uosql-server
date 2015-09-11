use std::vec::Vec;
use super::Error;
use super::types::SqlType;
use super::types::Column;
use std::io::{Write, Read, Seek};

#[derive(Debug)]
pub struct Rows <B: Write + Read + Seek> {
    data_src: B,
    columns: Vec<Column>,
}

/// Represents the lines read from file.
impl<B: Write + Read + Seek> Rows <B> {

    pub fn new(data_src: B, columns: &[Column]) -> Rows<B> {
        Rows { data_src: data_src,
               columns: columns.to_vec() }
    }

    /// reads the current row and writes the data into target_buf
    /// and moves the cursor to the beginnig of the next line which is not marked
    /// as deleted
    /// returns the bytes read
    pub fn read_row<W: Write>(&self, target_buf: &W) -> Result<u64, Error> {
       //let mut reader = (&mut self.data_src).take(10);
       Ok(0)
    }

    /// sets position before the first line
    pub fn reset_pos(&mut self) {
    }

    /// sets position to offset
    fn set_pos(&mut self, offset: u64) {

    }

    /// reads the header of the current row
    /// moves the cursor past the header
    /// returns an error if no RowHeader exists
    fn read_header(&mut self) -> Result <RowHeader, Error> {
        let mut target_buf = Vec::<u8>::new();
        try!(self.read_bytes(RowHeader::size(), &mut target_buf));
        Ok(RowHeader::new(target_buf[0]))
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

    fn read_bytes(&mut self, bytes_to_read: u64, mut target_buf: &mut Vec<u8>)
        -> Result<u64, Error>
    {
        let mut reader = (&mut self.data_src).take(bytes_to_read);
        let result = reader.read_to_end(&mut target_buf);
        let bytes_read = match result {
            Ok(n) => {
                if n as u64 != bytes_to_read {
                    return Err(Error::InterruptedRead);
                };
                n
            },
            Err(e) => {
                return Err(Error::Io(e));
            }
        };
        Ok(bytes_read as u64)
    }
}

pub struct RowHeader {
    data: u8,
}

impl RowHeader{

    pub fn new(data: u8) -> RowHeader {
        RowHeader {
            data: data,
        }
    }

    /// returns true if the current row is marked as deleted
    pub fn is_deleted(&self) -> bool {
        false
    }

    /// marks row as deleted
    pub fn set_deleted(&mut self) {

    }

    pub fn size() -> u64 {
        1
    }
}
