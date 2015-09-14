use std::vec::Vec;
use super::Error;
use super::types::SqlType;
use super::types::Column;
use std::io::{Write, Read, Seek, SeekFrom, Cursor};
use super::super::parse::ast::CompType;


#[derive(Debug)]
pub struct Rows <B: Write + Read + Seek> {
    data_src: B,
    columns: Vec<Column>,
    columns_size: u64,
    pub column_offsets: Vec<u64>,
    pos: u64,
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

        Rows {  data_src: data_src,
                columns: columns.to_vec(),
                columns_size: Self::get_columns_size(columns),
                column_offsets: column_offsets,
                pos: 0
            }
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
        Ok(target_vec.len() as u64)
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
        match self.data_src.seek(seek_from) {
            Ok(n) => {
                self.pos = n;
                Ok(n)
            },
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
    pub fn get_value(&self, row_data: &[u8], column_index: usize) -> Result<Vec<u8>, Error> {
        if row_data.len() == 0 {
            return Err(Error::InvalidState);
        }

        let s = self.column_offsets[column_index] as usize;
        let e = s + self.get_column(column_index).get_size() as usize;
        let d = row_data[s..e].to_vec();
        info!("get value: {:?}", d);
        Ok(d)
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
        self.pos += bytes_read as u64;
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
        let mut pks: Vec<usize> = Vec::new();
        let mut count: usize = 0;
        //let mut value: Vec<Vec<u8>> = Vec::new();
        // get pks
        {
            let mut it = self.columns.iter();
            loop {
                match it.next() {
                    Some(x) => {
                        if x.is_primary_key {
                            pks.push(count);
                        }
                        count += 1;
                    },
                    None => break,
                };
            }
        }
        // do lookups
        {
            let mut it = pks.iter();
            let first = match it.next() {
                Some(x) => x,
                None => return Err(Error::FoundNoPrimaryKey),
            };

            let val = try!(self.get_value(row_data, *first));
            let mut look = try!(self.lookup(*first, &val, CompType::Equ));

            loop {
                match it.next() {
                    Some(x) => {
                        let value = try!(self.get_value(row_data, *x));
                        look = try!(look.lookup(*x, &value, CompType::Equ));
                        if try!(look.is_empty()) {
                            break;
                        }
                    },
                    None => break,
                };
            }
            if !try!(look.is_empty()) {
                return Err(Error::PrimaryKeyValueExists);
            }
        }
        self.set_pos(SeekFrom::End(0));
        Ok(try!(self.add_row(row_data)))
    }

   pub fn lookup(&mut self, column_index: usize, value: &[u8], comp: CompType)
    -> Result<Rows<Cursor<Vec<u8>>>, Error>
    {
        let vec: Vec<u8> = Vec::new();
        let cursor = Cursor::new(vec);
        let mut rows = Rows::new(cursor, &self.columns);

        while true {
            let result = self.get_next_row(column_index, value, comp);
            match result {
                Ok(r) => {
                    println!{"Row: {:?}", r};
                    rows.add_row(&r);

                },
                Err(e) => {
                    match e {
                        Error::EndOfFile => {
                            info!("reached end of file");
                            break;
                        },
                        _ => return Err(e)
                    }
                }
            }
        }

        Ok(rows)
    }

    pub fn full_scan(&mut self) -> Result<Rows<Cursor<Vec<u8>>>, Error> {
        let vec: Vec<u8> = Vec::new();
        let cursor = Cursor::new(vec);
        let mut rows = Rows::new(cursor, &self.columns);
        let mut buf: Vec<u8> = Vec::new();

        while true {
            match self.next_row(&mut buf) {
                Ok(_) => {
                        rows.add_row(& buf);
                },
                Err(e) => {
                    match e {
                        Error::EndOfFile => break,
                        _ => return Err(e)
                    }
                },
            }
        }
        Ok(rows)
    }

    pub fn is_empty(&mut self) -> Result <bool, Error> {
        //data_src.is_empty()
        let old_pos = self.pos;

        let result: Result<bool, Error> = match self.set_pos(SeekFrom::End(0)) {
            Ok(n) => {
                if n == 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            Err(e) => {
                Err(e)
            },
        };

        self.pos = old_pos;

        result
    }

    /// moves the cursor from the current position to the first row
    /// which fulfills the constraint. Use next_row to read that row.
    pub fn get_next_row(&mut self, column_index: usize, value: &[u8], comp: CompType)
    -> Result<Vec<u8>, Error>
    {
        let mut row = Vec::<u8>::new();
        let mut b = true;

        while b {
            b = match self.next_row(&mut row) {
                Ok(_) => {
                    let col = self.get_column(column_index);

                    let row_value: &Vec<u8> = &try!(self.get_value(&row, column_index));
                    let cmp_result = try!(col.sql_type.cmp(row_value, value, comp));

                    if cmp_result {
                        false
                    } else {
                        row.clear();
                        true
                    }
                },
                Err(e) => {
                    return Err(e)
                }
            }
        }

        Ok(row)
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
