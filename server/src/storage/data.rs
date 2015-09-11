use std::vec::Vec;
use super::Error;
use super::types::SqlType;
use super::types::Column;
use std::io::{Write, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Rows <B: Write + Read + Seek> {
    data_src: B,
    columns: Vec<Column>,
    columns_size: u64,
}

/// Represents the lines read from file.
impl<B: Write + Read + Seek> Rows <B> {

    pub fn new(data_src: B, columns: &[Column]) -> Rows<B> {
        Rows { data_src: data_src,
               columns: columns.to_vec(),
               columns_size: Self::get_columns_size(columns)}
    }

    fn get_columns_size(columns: &[Column]) -> u64 {
        let mut size: u64 = 0;
        for c in columns {
            size += c.get_size() as u64;
        }
        size
    }

    /// reads the next row, which is not marked as deleted
    /// and writes the data into target_buf
    /// returns the bytes read or an Error otherwise.
    pub fn next_row<W: Write>(&mut self, mut target_buf: &mut W)
        -> Result<u64, Error>
    {
        let mut target_vec = Vec::<u8>::new();
        let columns_size = self.columns_size;

        let mut row_header: RowHeader = try!(self.read_header());

        while row_header.is_deleted() {
            self.skip_row();
            row_header = try!(self.read_header());
        }

        try!(self.read_bytes(columns_size, &mut target_vec));
        try!(target_buf.write_all(&target_vec));
        Ok(target_vec.len() as u64)
    }

    pub fn skip_row(&mut self) -> Result<u64, Error> {
        let columns_size = self.columns_size as i64;
        self.set_pos(SeekFrom::Current(columns_size))
    }

    /// sets position before the first line
    pub fn reset_pos(&mut self) -> Result<u64, Error> {
        self.set_pos(SeekFrom::Start(0))
    }

    /// sets position to offset
    fn set_pos(&mut self, seek_from: SeekFrom) -> Result<u64, Error> {
        match self.data_src.seek(seek_from) {
            Ok(n) => Ok(n),
            Err(e) => return Err(Error::Io(e))
        }
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

    /// reads the specified amount of bytes and writes them into target_buf
    /// returns bytes_read or error if bytes_read != bytes_to_read
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
