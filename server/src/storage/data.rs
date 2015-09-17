use std::vec::Vec;
use super::Error;
use super::types::Column;
use std::io::{Write, Read, Seek, SeekFrom, Cursor};
use super::super::parse::ast::CompType;

#[derive(Debug)]
pub struct Rows <B: Write + Read + Seek> {
    data_src: B,
    pub columns: Vec<Column>,
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
    /// returns the sum of the column sizes
    fn get_columns_size(columns: &[Column]) -> u64 {
        let mut size: u64 = 0;
        for c in columns {
            size += c.get_size() as u64;
        }
        size
    }

    /// Returns the last not deleted row.
    /// The search begins with the first row located before position.
    /// An error is returned if no row could be found.
    /// Moves cursor to the end of the read row.
    fn get_last_row(&mut self, position: SeekFrom, ignore_deleted_rows: bool)
        -> Result <Vec<u8>, Error>
    {
        try!(self.set_pos(position));
        try!(self.prev_row());

        // Moves cursor to the beginning of the first not deleted row
        // from the end.
        while ignore_deleted_rows && try!(self.is_next_row_deleted(true)) {
            try!(self.prev_row());
        }

        let mut row_data = Vec::<u8>::new();
        try!(self.next_row_ex(&mut row_data, false));
        Ok(row_data)
    }

    /// Tests if the row after the cursor is marked as deleted
    /// restore_cursor: if true, calling the function will move the cursor,
    /// if false, the cursor will be moved by row_header.size() bytes.
    fn is_next_row_deleted(&mut self, restore_cursor: bool)
        -> Result <bool, Error>
    {
        info!("Reading Header");
        let row_header: RowHeader = try!(self.read_header());
        info!("Header Read");
        if restore_cursor {
            try!(self.set_pos(SeekFrom::Current(-(RowHeader::size() as i64))));
        };
        Ok(row_header.is_deleted())
    }

    /// Moves the cursor to the beginning of the next deleted row
    fn move_to_next_deleted_row(&mut self) -> Result<(), Error> {
        let mut row_data = Vec::<u8>::new();
        while try!(self.is_next_row_deleted(true)) {
            try!(self.next_row_ex(&mut row_data, false));
        }
        Ok(())
    }

    /// reads the next row, which is not marked as deleted
    /// and writes the data into target_buf
    /// returns the bytes read or an Error otherwise.
    /// Returns Error:EndOfFile if no next row could be read.
    pub fn next_row<W: Write>(&mut self, mut target_buf: &mut W)
        -> Result<u64, Error>
    {
        self.next_row_ex(target_buf, true)
    }

    /// reads the next row, which is not marked as deleted
    /// and writes the data into target_buf
    /// returns the bytes read or an Error otherwise.
    /// Returns Error:EndOfFile if no next row could be read.
    /// if ignore_deleted_rows == true, deleted rows will be skipped.
    fn next_row_ex<W: Write>(&mut self, mut target_buf: &mut W, ignore_deleted_rows: bool)
        -> Result<u64, Error>
    {
        info!("Moving to next row.");
        let mut target_vec = Vec::<u8>::new();
        let columns_size = self.columns_size;

        while try!(self.is_next_row_deleted(false)) && ignore_deleted_rows {
            try!(self.skip_row());
        }

        try!(self.read_bytes(columns_size, &mut target_vec));
        try!(target_buf.write_all(&target_vec));
        info!("Row written to target_buf. Leaving next_row.");
        Ok(target_vec.len() as u64)
    }

    /// Sets pos to the beginning of the next row
    /// Be sure to only call skip_row after the row header was
    /// read.
    fn skip_row(&mut self) -> Result<u64, Error> {
        let columns_size = self.columns_size as i64;
        self.set_pos(SeekFrom::Current(columns_size))
    }

    /// Returns the size of a row including the size of the
    /// row header.
    fn get_row_size(&self) -> u64 {
        self.columns_size + RowHeader::size()
    }

    /// sets pos to the beginning of the previous row
    /// Returns the new positino of the cursor.
    /// Returns Error::BeginningOfFile if cursor would be moved to a negative
    /// offset.
    fn prev_row(&mut self) -> Result<u64, Error> {
        let row_size = self.get_row_size() as i64;
        if self.pos as i64 - row_size < 0 {
            return Err(Error::BeginningOfFile);
        };
        self.set_pos(SeekFrom::Current(-row_size))
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

    /// set delete bit for one row
    fn delete_row(&mut self) -> Result<(), Error> {
        try!(self.prev_row());
        let row_header = RowHeader::new(1);
        try!(self.write_bytes(&row_header.to_raw_data()));
        try!(self.skip_row());
        info!("Row Deleted");
        Ok(())
    }

    /// Reorganizes the current instance
    /// Removes all rows which are marked as deleted.
    /// Returns the new size of the reorganized object.
    pub fn reorganize(&mut self) -> Result<u64, Error> {
        try!(self.reset_pos());
        let mut move_result: Result<(), Error>;
        let mut saved_pos;
        let mut last_row_found: bool;
        let mut row_data: Vec<u8> = Vec::<u8>::new();
        let mut last_row_seek_from = SeekFrom::End(0);

        loop {
            // find first deleted row
            move_result = self.move_to_next_deleted_row();
            match move_result {
                Ok(_) => { },
                Err(Error::EndOfFile) => break, // No deleted row found.
                Err(e) => return Err(e)
            };

            saved_pos = self.pos; // pos before deleted row.
            last_row_found = match self.get_last_row(last_row_seek_from, true) {
                Err(Error::BeginningOfFile) => false, // No not deleted row found.
                Err(e) => return Err(e),
                Ok(v) => {
                    row_data = v;
                    true
                }
            };
            // pos is located after the row returned by the previous call
            // to self.get_last_row().
            // deleted row found && last_row found
            if last_row_found {
                if self.pos == saved_pos {
                    // Cursor is located after the first not deleted line.
                    return Ok(self.pos);
                };
                last_row_seek_from =
                    SeekFrom::Start(self.pos - self.get_row_size());
                try!(self.set_pos(SeekFrom::Start(saved_pos)));
                try!(self.add_row(&row_data));
            }
            else {
                // gelöschte Zeile gefunden && !last_row found
                try!(self.set_pos(SeekFrom::Start(saved_pos)));
                return Ok(self.pos);
            }
        };
        Err(Error::InvalidState)
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

    /// Sets value of column_index' column to new_value.
    fn set_value(&self, row_data: &mut[u8], new_value: &[u8], column_index: usize)
    {
        // start index of column
        let s = self.column_offsets[column_index] as usize;
        // end index of column
        let e = s + self.get_column(column_index).get_size() as usize;
        let mut c = 0;
        for i in s..e {
            row_data[i] = new_value[c];
            c += 1;
        }
    }

    /// returns the columns
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

    /// writes data to data_src
    /// returns bytes written
    fn write_bytes(&mut self, data: &[u8]) -> Result<u64, Error> {
            match self.data_src.write_all(data) {
            Ok(_) => {
                return Ok(data.len() as u64)
            },
            Err(e) => return Err(Error::Io(e))
        }
    }

    /// Inserts a new row with row_data. Does not check if the primary key exists.
    /// Returns the number of rows_inserted.
    pub fn insert_row_without_primary(&mut self, row_data: &[u8]) -> Result<u64, Error> {
        try!(self.set_pos(SeekFrom::End(0)));
        Ok(try!(self.add_row(row_data)))
    }

    /// Inserts a new row with row_data.
    /// Returns the number of rows inserted.
    pub fn insert_row(&mut self, row_data: &[u8]) -> Result<u64, Error> {
        let mut pks: Vec<usize> = Vec::new();
        let mut count: usize = 0;
        // get pks
        info!("getting primary keys ....");
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
        info!("doing lookups to search for matches ....");
        {
            let mut it = pks.iter();
            let first = match it.next() {
                Some(x) => x,
                None => return Err(Error::FoundNoPrimaryKey),
            };

            let val = try!(self.get_value(row_data, *first));
            let mut look = try!(self.lookup(*first, (&val, None), CompType::Equ));

            loop {
                match it.next() {
                    Some(x) => {
                        let value = try!(self.get_value(row_data, *x));
                        look = try!(look.lookup(*x, (&value, None), CompType::Equ));
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
        try!(self.set_pos(SeekFrom::End(0)));
        Ok(try!(self.add_row(row_data)))
    }

    /// deletes rows which fulfills a constraint
    /// return rows deleted
    pub fn delete(&mut self, column_index: usize, value: (&[u8], Option<usize>), comp: CompType)
     -> Result<u64, Error>
    {
        try!(self.reset_pos());
        let mut count = 0;
        loop {
            match self.get_next_row(column_index, value, comp) {
                Ok(_) => {
                    count += 1;
                    try!(self.delete_row());
                },
                Err(Error::EndOfFile) => {
                    info!("reached end of file");
                    break;
                },
                Err(e) => return Err(e)
            }
        }
        Ok(count)
    }

    /// Updates all rows fulfilling the constraint.
    /// constraint_column_index: index of the rows whose value is compared
    /// to constraint_value.
    /// values[(column_index: usize, new_value: &[u8])]: Slice of tuples
    /// The first value of the tuple contains the index of the column to be
    /// updated. The second value contains the new value for the column.
    /// returns the numer of rows updated.
    /// Panics if a row could not be updated. If modify fails, the updated row
    /// will be lost.
    pub fn modify(&mut self, constraint_column_index: usize,
     constraint_value: (&[u8], Option<usize>), comp: CompType,
     values: &[(usize, &[u8])] )-> Result<u64, Error>
    {
        info!("Modify rows values {:?}", values);
        let mut rows = try!(self.lookup(constraint_column_index,
                                        constraint_value,
                                        comp));
        try!(rows.reset_pos());
        let primary_key_index = self.get_primary_key_column_index();
        let mut primary_key_value: Vec<u8>;
        let mut row_data = Vec::<u8>::new();
        let mut result = rows.next_row(&mut row_data);
        let mut rows_deleted: u64;
        let mut updated_rows: u64 = 0;

        if primary_key_index == constraint_column_index {
            return Err(Error::PrimaryKeyNotAllowed);
        }
        // loop through rows.
        loop {
            match result {
                Ok(_) => { },
                Err(Error::EndOfFile) => {
                    break;
                },
                Err(e) => return Err(e)
            };

            primary_key_value = try!(rows.get_value(&row_data,
                                                    primary_key_index));

            rows_deleted = try!(self.delete(primary_key_index,
                                            (&primary_key_value, None),
                                            CompType::Equ));

            if rows_deleted != 1 { panic!("Exactly one row should have been
                                           deleted! database inconsistent") };

            for kvp in values {
              self.set_value(&mut row_data,
                             &kvp.1, // new_value
                             kvp.0); // column_index
            }

            match self.insert_row_without_primary(&row_data) {
                Ok(_) => { updated_rows += 1 },
                Err(e) => {
                    panic!("Modify failed. Deleted old row but could not insert
                            modified new row. Returned error: {:?}", e);
                }
            }

            row_data.clear();
            result = rows.next_row(&mut row_data);
        }
        info!("rows modified");
        Ok(updated_rows)
    }

    /// Returns an new Rows object with all rows which fulfill the constraint.
    /// column_index: index of the column whose value should match value
    /// comp: defines how the value of the column and value should be compared.
    pub fn lookup(&mut self, column_index: usize, value: (&[u8], Option<usize>), comp: CompType)
        -> Result<Rows<Cursor<Vec<u8>>>, Error>
    {
        try!(self.reset_pos());
        let vec: Vec<u8> = Vec::new();
        let cursor = Cursor::new(vec);
        let mut rows = Rows::new(cursor, &self.columns);
        info!("starting lookup for column {:?}", column_index);
        loop {
            let result = self.get_next_row(column_index, value, comp);
            match result {
                Ok(r) => {
                    info!{"Row: {:?}", r};
                    try!(rows.add_row(&r));
                },
                Err(Error::EndOfFile) => {
                    info!("reached end of file");
                    break;
                }
                Err(e) => return Err(e)
            }
        }
        Ok(rows)
    }

    /// scans the entire file
    /// returns all rows which are not deleted
    pub fn full_scan(&mut self) -> Result<Rows<Cursor<Vec<u8>>>, Error> {
        try!(self.reset_pos());
        let vec: Vec<u8> = Vec::new();
        let cursor = Cursor::new(vec);
        let mut rows = Rows::new(cursor, &self.columns);
        let mut buf: Vec<u8> = Vec::new();
        info!("starting full scan ....");
        loop {
            match self.next_row(&mut buf) {
                Ok(_) => {
                        try!(rows.add_row(& buf));
                        info!(".");
                        //info!("buf: {:?}", &buf);
                        buf.clear();
                },
                Err(Error::EndOfFile) => break,
                Err(e) => { return Err(e) }
            }
        }
        info!("finished full scan ....");
        Ok(rows)
    }

    /// checks if object is containing rows
    /// returns bool on success else Error
    pub fn is_empty(&mut self) -> Result <bool, Error> {
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

    /// moves the cursor from the current position the row after the first row
    /// which fulfills the constraint.
    /// Returns the first row which fulfilly the condition.
    pub fn get_next_row(&mut self, column_index: usize, value: (&[u8], Option<usize>), comp: CompType)
    -> Result<Vec<u8>, Error>
    {
        let mut row = Vec::<u8>::new();
        let mut b = true;

        while b {
            b = match self.next_row(&mut row) {
                Ok(_) => {
                    let col = self.get_column(column_index);

                    let row_value: &Vec<u8> = &try!(self.get_value(&row, column_index));
                    if value.1.is_none() {
                        let cmp_result = try!(col.sql_type.cmp(row_value, value.0, comp));

                        if cmp_result {
                            false
                        } else {
                            row.clear();
                            true
                        }
                    } else {
                        let cmpindex = value.1.unwrap();
                        let cmp_value: &Vec<u8> = &try!(self.get_value(&row, cmpindex));
                        let cmp_result = try!(col.sql_type.cmp(row_value, cmp_value, comp));
                        if cmp_result {
                            false
                        } else {
                            row.clear();
                            true
                        }

                    }
                },
                Err(e) => {
                    return Err(e)
                }
            }
        }
        Ok(row)
    }

    /// Returns the index of the column which is the primary key.
    fn get_primary_key_column_index(&self) -> usize {
        let mut column_count = 0;
        for column in &self.columns {
            if column.is_primary_key == true {
                return column_count;
            }
            column_count += 1;
        }
        column_count
    }

    /// Returns a new ResultSet containing all rows of the current object
    pub fn to_result_set(&mut self) -> Result<ResultSet, Error> {
        try!(self.reset_pos());
        let mut data = Vec::<u8>::new();
        let mut row_data;
        let mut result:Result<u64, Error>;

        loop {
            row_data = Vec::<u8>::new();
            result = self.next_row(&mut row_data);
            match result {
                Ok(_) => { },
                Err(Error::EndOfFile) => {
                    break;
                },
                Err(e) => return Err(e)
            };
            data.extend(row_data.into_iter())
        }

        Ok(ResultSet { data: data, columns: self.columns.clone() })
    }
}

/// Representation of a RowHeader
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

    /// Returns the bytes of the RowHeader
    pub fn to_raw_data(&self) -> Vec<u8> {
        let mut raw_data = Vec::<u8>::new();
        raw_data.push(self.data);
        raw_data
    }
}

/// Encodable and decodable representation of a Rows object
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct ResultSet {
    pub data: Vec<u8>,
    pub columns: Vec<Column>,
}
