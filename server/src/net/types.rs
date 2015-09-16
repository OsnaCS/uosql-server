/// Because of cyclic references to modules we need to use super::Error to use
/// the enum. Nightly Build supports using enums - so we can fix super::Error in
/// about 3 months ;)

use std::error::Error;
use storage::ResultSet;
use storage::{Column, SqlType};
use storage::types::FromSql;
use std::cmp::{max};

/// Representation of a ResultSet with its useful functions to get data.
pub struct DataSet {
    data: Vec<Vec<Vec<u8>>>,
    columns: Vec<Column>,
    current_pos : usize,
    line_cnt: usize
}

impl DataSet {

    pub fn get_col_cnt (&self) -> usize {
        self.columns.len()
    }

    pub fn data_empty (&self) -> bool {
        if self.data.len() == 0 {
            return true
        }
        false
    }
    pub fn metadata_empty(&self) -> bool {
        if self.columns.len() == 0 {
            return true
        }
        false
    }

    pub fn get_col_idx (&self, name: String) -> Option<usize> {
        for i in 0..self.columns.len() {
            if self.columns[i].name == name {
                return Some(i)
            }
        }
        None
    }

    pub fn get_col_name (&mut self, idx: usize) -> Option<&str> {
        if idx >= self.columns.len() { // idx out of bounds
            None
        } else {
            Some(&self.columns[idx].name)
        }
    }

    pub fn get_type_by_name (&mut self, name: String) -> Option<SqlType> {
        match self.get_col_idx (name) {
            Some(idx) => Some(self.columns[idx].sql_type),
            None => None
        }
    }

    pub fn get_is_primary_by_name (&mut self, name: String) -> Option<bool> {
        match self.get_col_idx (name) {
            Some(idx) => Some(self.columns[idx].is_primary_key),
            None => None
        }
    }

    pub fn get_allow_null_by_name (&mut self, name: String) -> Option<bool> {
        match self.get_col_idx (name) {
            Some(idx) => Some(self.columns[idx].allow_null),
            None => None
        }
    }

    pub fn get_description_by_name (&mut self, name: String) -> Option<&str> {
        match self.get_col_idx (name) {
            Some(idx) => Some(&self.columns[idx].description),
            None => None
        }
    }

    pub fn get_type_by_idx (&mut self, idx: usize) -> Option<SqlType> {
        if idx >= self.columns.len() { //idx out of bounds
            None
        } else {
            Some(self.columns[idx].sql_type)
        }
    }

    pub fn get_is_primary_key_by_idx (&mut self, idx: usize) -> Option<bool> {
        if idx >= self.columns.len() { //idx out of bounds
            None
        } else {
            Some(self.columns[idx].is_primary_key)
        }
    }

    pub fn get_allow_null_by_idx (&mut self, idx: usize) -> Option<bool> {
        if idx >= self.columns.len() { //idx out of bounds
            None
        } else {
            Some(self.columns[idx].allow_null)
        }
    }

    pub fn get_description_by_idx (&mut self, idx: usize) -> Option<&str> {
        if idx >= self.columns.len() { //idx out of bounds
            None
        } else {
            Some(&self.columns[idx].description)
        }
    }

    /// Return next data entry. next() has to be called first it initialize
    /// the pointer
    pub fn next_int_by_idx (&mut self, idx: usize) -> Option<i32> {
        if idx >= self.columns.len() { //idx out of bounds
            None
        } else {
            match i32::from_sql(&self.data[self.current_pos - 1][idx][..]) {
                Ok(val) => Some(val),
                Err(e) => {println!("int by idx: {:?}", e); None}
            }
        }
    }

    /// Return next data entry. next() has to be called first it initialize
    /// the pointer
    pub fn next_bool_by_idx (&mut self, idx: usize) -> Option<bool> {
        if idx >= self.columns.len() { //idx out of bounds
            None
        } else {
            match bool::from_sql(&self.data[self.current_pos - 1][idx][..]) {
                Ok(val) => Some(val),
                Err(e) => {println!("bool by idx: {:?}", e); None}
            }
        }
    }

    /// Return next data entry. next() has to be called first it initialize
    /// the pointer
    pub fn next_char_by_idx (&mut self, idx: usize) -> Option<String> {
        if idx >= self.columns.len() { //idx out of bounds
            None
        } else {
            // find the first pos that does not contain '0' value
            let mut pos = 0;
            let data = &self.data[self.current_pos - 1][idx][..];
            while pos < self.columns[idx].sql_type.size() as usize {
                if data[pos] == 0 {
                    break;
                }
                pos += 1;
            }
            match String::from_sql(&self.data[self.current_pos - 1][idx][0..pos]) {
                Ok(val) => { Some(val) },
                Err(e) => { None }
            }
        }
    }

    /// Return next data entry. next() has to be called first it initialize
    /// the pointer
    pub fn next_int_by_name (&mut self, name: String) -> Option<i32> {
        match self.get_col_idx (name) {
            Some(idx) => self.next_int_by_idx (idx),
            None => None
        }
    }

    /// Return next data entry. next() has to be called first it initialize
    /// the pointer
    pub fn next_bool_by_name (&mut self, name: String) -> Option<bool> {
        match self.get_col_idx (name) {
            Some(idx) => self.next_bool_by_idx (idx),
            None => None
        }
    }

    /// Return next data entry. next() has to be called first it initialize
    /// the pointer
    pub fn next_char_by_name (&mut self, name: String) -> Option<String> {
        match self.get_col_idx (name) {
            Some(idx) => self.next_char_by_idx (idx),
            None => None
        }
    }

    /// Set the data pointer before the first entry (pos = -1). next() has to be
    /// called first to start a new next... - loop
    pub fn first (&mut self) {
        self.current_pos = 0
    }

    /// Set the data pointer after the last entry . previous() has to be called
    /// first to start a new backward loop
    pub fn last (&mut self) {
        self.current_pos = self.line_cnt
    }

    /// Move the pointer to the next line. Return false if end of data, else true.
    pub fn next(&mut self) -> bool {
        if self.current_pos >= self.line_cnt  {
            false
        } else {
            self.current_pos += 1;
            true
        }
    }

    /// Move the pointer to the previous line. Return false if end of data, else true.
    pub fn previous (&mut self) -> bool {
        if self.current_pos == 0 {
            false
        } else {
            self.current_pos -= 1;
            true
        }
    }
}

/// Sort the Vec<u8> data into DataSet for further use.
pub fn preprocess (data: &ResultSet) -> DataSet {
    let col_count = data.columns.len();
    let data_len = data.data.len();
    // get line length
    let mut line_len = 0;
    let mut arr = Vec::<u32>::new();
    for i in 0..(col_count) {
        line_len += data.columns[i].get_size();
        arr.push(data.columns[i].get_size());
    }
    // number of lines
    if line_len == 0 {
        return DataSet {data: Vec::new(), columns: data.columns.clone(),
                    current_pos: 0, line_cnt: 0}
    }

    let line_count = data_len / line_len as usize;
    let mut process_data = Vec::new();

    // split data
    let mut pos = 0;
    for i in 0..(line_count) {
        let mut colvec = Vec::new();
        for j in 0..(col_count) {
            let mut linevec = Vec::<u8>::new();
            for _ in 0..(arr[j]) {
                linevec.push(data.data[pos]);
                pos += 1;
            }
            colvec.push(linevec);   // push the single data vec to column
        }
        process_data.push(colvec);
    }
    // println!("data = {:?}", data);
    // println!("process data = {:?}", process_data);
    DataSet {data:process_data, columns: data.columns.clone(),
                    current_pos: 0, line_cnt: line_count}
}

/// Code numeric value sent as first byte
#[derive(PartialEq, RustcEncodable, RustcDecodable)]
#[repr(u8)]
pub enum PkgType {
    Greet = 0,
    Login,
    Command,
    Error,
    Ok,
    Response,
    AccDenied,
    AccGranted,
}

/// Struct to send the kind of error and error message to the client
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct ClientErrMsg {
    code: u16,
    pub msg: String
}

/// Convert the possible Error to a serializable ClientErrMsg struct
impl From<super::Error> for ClientErrMsg {
    fn from(error: super::Error) -> ClientErrMsg {
        match error {
            super::Error::Io(_) => ClientErrMsg {
                code: 0,
                msg: error.description().into()
            },
            super::Error::UnexpectedPkg => ClientErrMsg {
                code: 2,
                msg: error.description().into()
            },
            super::Error::UnknownCmd => ClientErrMsg {
                code: 3,
                msg: error.description().into()
            },
            super::Error::Encode(_) => ClientErrMsg {
                code: 4,
                msg: error.description().into()
            },
            super::Error::Decode(_) => ClientErrMsg {
                code: 5,
                msg: error.description().into()
            },
            super::Error::UnEoq(_) => ClientErrMsg {
                code: 6,
                msg: error.description().into()
            }
        }
    }
}

/// This is the first packet being sent by the server after the TCP connection
/// is established.
#[derive(RustcEncodable, RustcDecodable)]
pub struct Greeting {
    pub protocol_version: u8,   // 1 byte
    pub message: String,        // n bytes
}

impl Greeting {
    pub fn make_greeting(version: u8, msg: String) -> Greeting {
        Greeting { protocol_version: version, message: msg }
    }
}

/// The client responds with this packet to a `Greeting` packet, finishing the
/// authentication handshake.
#[derive(Default, RustcEncodable, RustcDecodable)]
pub struct Login {
    pub username: String,
    pub password: String
}

/// Sent by the client to the server.
///
/// Many commands are executed via query, but there are some "special"
/// commands that are not sent as query.
#[derive(RustcEncodable, RustcDecodable, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    Quit,
    Ping,
    Query(String),
    // Shutdown,
    // Statistics,
}
