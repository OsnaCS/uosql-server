/// Because of cyclic references to modules we need to use super::Error to use
/// the enum. Nightly Build supports using enums - so we can fix super::Error in
/// about 3 months ;)

use std::error::Error;
use storage::ResultSet;
use storage::Column;



pub struct DataSet {
    data: Vec<Vec<Vec<u8>>>,
    columns: Vec<Column> 
}

impl DataSet {
    
}

pub fn preprocess (data: &ResultSet) -> () {
    let col_count = data.columns.len();
    println!("col_count = {:?}", col_count);
    let data_len = data.data.len();
    println!(" data_len = {:?}", data_len);
    // get line length
    let mut line_len = 0;
    let mut arr = Vec::<u32>::with_capacity(col_count as usize);
    for i in 0..(col_count) {
        println!("col count pos: {}", i);
        line_len += data.columns[i].get_size();
        arr[i] = data.columns[i].get_size();
    }
    println!("{}", "somethisng" );
    // number of lines
    let line_count = data_len / line_len as usize;

    let mut process_data = Vec::with_capacity(col_count as usize);

    // initiate the data columns
    for i in 0..(col_count) {
        println!("col count pos2: {}", i);
        process_data.push(Vec::<Vec<u8>>::with_capacity(line_count));
    }

    // split data
    let mut pos = 0;
    for i in 0..(line_count) {
        println!("line count 2: {}", i);
        for j in 0..(col_count) {
            println!("col count 3: {}", j);
            for k in 0..(arr[j]) {
                println!("k: {}, pos = {}", k, pos);
                process_data[j][i].push(data.data[pos] as u8);
                pos += 1;
            }
        }
    }
    println!("{:?}", process_data);
    return ()

}

    // pub fn get_sql_type(&self) -> &SqlType {
    //     &self.sql_type
    // }

    // pub fn get_column_name(&self) -> &str {
    //     &self.name
    // }

    // pub fn get_size(&self) -> u32 {
    //     self.sql_type.size() as u32
    // }

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
