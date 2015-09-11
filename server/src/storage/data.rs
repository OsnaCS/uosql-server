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
    pub current_row: Vec<u8>,
    pub column_offsets: Vec<u64>,
}

/// Represents the lines read from file.
impl<B: Write + Read + Seek> Rows <B> {

    pub fn new(data_src: B, columns: &[Column]) -> Rows<B> {
        let mut column_offsets = Vec::<u64>::new();
        let mut offset: u64 = 0;
        for c in columns {
            column_offsets.push(offset);
            offset += c.get_size() as u64;
        }

        Rows { data_src: data_src,
               columns: columns.to_vec(),
               columns_size: Self::get_columns_size(columns),
               current_row: Vec::<u8>::new(),
               column_offsets: column_offsets }
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
        info!("Reading Header");
        let mut row_header: RowHeader = try!(self.read_header());
        info!("Header Read");
        while row_header.is_deleted() {
            self.skip_row();
            row_header = try!(self.read_header());
        }

        info!("data read");
        try!(self.read_bytes(columns_size, &mut target_vec));
        try!(target_buf.write_all(&target_vec));
        info!("data read");
        self.current_row = target_vec;
        Ok(self.current_row.len() as u64)
    }

    /// sets pos to the beginning of the next row
    fn skip_row(&mut self) -> Result<u64, Error> {
        let columns_size = self.columns_size as i64;
        self.set_pos(SeekFrom::Current(columns_size))
    }

    /// sets pos to the beginning of the previous row
    fn prev_row(&mut self) -> Result<u64, Error> {
        let columns_size = self.columns_size as i64;
        self.set_pos(SeekFrom::Current(-(columns_size+(RowHeader::size()) as i64)))
    }

    /// sets position before the first line
    pub fn reset_pos(&mut self) -> Result<u64, Error> {
        self.set_pos(SeekFrom::Start(0))
    }

    /// sets position to offset
    fn set_pos(&mut self, seek_from: SeekFrom) -> Result<u64, Error> {
        self.current_row.clear();
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
    pub fn add_row(&mut self, data: &[u8]) -> Result<u64, Error> {
        info!("Adding Row");
        let new_row_header = RowHeader::new(0);
        try!(self.write_bytes(&new_row_header.to_raw_data()));
        Ok(try!(self.write_bytes(&data)))
    }

    pub fn delete_row(&mut self) -> Result<(), Error> {
        self.prev_row();
        let row_header = RowHeader::new(1);
        try!(self.write_bytes(&row_header.to_raw_data()));
        self.skip_row();
        Ok(())
    }

    /// returns the value of the column_index' column of the current row
    /// returns Error::InvalidState if no current row exists
    pub fn get_value(&self, column_index: usize) -> Result<Vec<u8>, Error> {
        if self.current_row.len() == 0 {
            return Err(Error::InvalidState);
        }

        let s = self.column_offsets[column_index] as usize;
        let e = s + self.get_column(column_index).get_size() as usize;
        Ok(self.current_row[s..e].to_vec())
    }

    pub fn get_column(&self, index: usize) -> &Column {
        &self.columns[index]
    }

    /// reads the specified amount of bytes and writes them into target_buf
    /// returns bytes_read or error if bytes_read != bytes_to_read
    fn read_bytes(&mut self, bytes_to_read: u64, mut target_buf: &mut Vec<u8>)
        -> Result<u64, Error>
    {
        info!("Reading Bytes");
        let mut reader = (&mut self.data_src).take(bytes_to_read);
        let result = reader.read_to_end(&mut target_buf);
        let bytes_read = match result {
            Ok(n) => {
                if n == 0 {
                    return Err(Error::EndOfFile);
                };
                if n as u64 != bytes_to_read {
                    info!("bytes_read {:?} bytes_to_read {:?}", n, bytes_to_read);
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

    fn write_bytes(&mut self, data: &[u8]) -> Result<u64, Error> {
            match self.data_src.write_all(data) {
            Ok(_) => {
                return Ok(data.len() as u64)
            },
            Err(e) => return Err(Error::Io(e))
        }
    }

    /// Inserts a new row with row_data.
    /// Returns the number of rows inserted.
    pub fn insert_row(&mut self, row_data: &[u8]) -> Result<u64, Error> {
        self.set_pos(SeekFrom::End(0));
        try!(self.add_row(row_data));
        Ok(1)
    }
}

pub struct RowHeader {
     pub data: u8,
}

impl RowHeader{

    pub fn new(deleted: u8) -> RowHeader {
        let data = 0 as u8;
        let mut h = RowHeader {
            data: data,
        };
        h.set_deleted(deleted);
        h
    }

    /// returns true if the current row is marked as deleted
    pub fn is_deleted(&self) -> bool {
        info!("check for delete bit");
        (self.data & 1 as u8) == 1
    }

    /// marks row as deleted
    pub fn set_deleted(&mut self, value: u8) {
        info!("set delete bit");
        if value == 1 {
            self.data |= 1 as u8
        }
        else {
            self.data &= 0xFE as u8
        }
    }
    /// returns size of RowHeader
    pub fn size() -> u64 {
        1
    }

    pub fn to_raw_data(&self) -> Vec<u8> {
        let mut raw_data = Vec::<u8>::new();
        raw_data.push(self.data);
        raw_data
    }
}
