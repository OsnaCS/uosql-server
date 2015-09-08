use super::{Error};
use std::io::Write;
use super::super::parse::ast::DataSrc;
use byteorder::{BigEndian, WriteBytesExt};


/// General enums in SQL
#[derive(Debug, Clone, Copy, RustcDecodable, RustcEncodable)]
pub enum SqlType {
    Int,
    Bool,
    Char(u8),
    VarChar(u16)
}


/// Defines the size of Sql data types
/// and returns them
impl SqlType {
    pub fn size(&self) -> u32 {
        match self {
            &SqlType::Int => 4 as u32,
            &SqlType::Bool => 1 as u32,
            &SqlType::Char(len) => (len) as u32,
            &SqlType::VarChar(len) => (len) as u32
        }
    }

    /// Writes data to buf
    /// Returns the bytes written.
    /// Returns Error::InvalidType if type of DataSrc does not match expected
    /// type.
    /// Returns byteorder::Error, if data could not be written to buf.

    pub fn encode_into<W: Write>(&self, mut buf: &mut W, data: &DataSrc)
    -> Result<u32, Error>
    {
        match self {
            &SqlType::Int => {
                match data {
                    &DataSrc::Int(a) => {
                        if a > i32::max_value() as i64 {
                            Err(Error::InvalidType)
                        }
                        else {
                            try!(buf.write_i32::<BigEndian>(a as i32));
                            Ok(self.size())
                        }
                    },
                    _=> {
                        Err(Error::InvalidType)
                    }
                }
            },
            &SqlType::Bool => {
                match data {
                    &DataSrc::Bool(a) => {
                        try!(buf.write_u8(a as u8));
                        Ok(self.size())
                    }
                    _=> {
                        Err(Error::InvalidType)
                    }
                }
            },
            &SqlType::Char(len) => {
                match data {
                    &DataSrc::String(ref a) => {
                        let str_as_bytes = Self::to_bytes(&a, len as u32);
                        Ok(try!(Self::write_to_buf(buf, &str_as_bytes)))
                    }
                    _=> {
                        Err(Error::InvalidType)
                    }
                }
            },
            &SqlType::VarChar(len) => {
                match data {
                    &DataSrc::String(ref a) => {
                        let str_as_bytes = Self::to_bytes(&a, len as u32);
                        Ok(try!(Self::write_to_buf(buf, &str_as_bytes)))
                    }
                    _=> {
                        Err(Error::InvalidType)
                    }
                }
            }
        }
    }
    /// Writes the vector vec to buf.
    /// Returns the bytes written.
    /// Returns byteorder::Error if vec could not be written to buf
    fn write_to_buf<W: Write>(mut buf: W, vec: &Vec<u8>) -> Result<u32, Error> {
        let mut it = vec.into_iter();
        let mut bytes_written = 0;

        loop {
            match it.next() {
                Some(v) => {
                    try!(buf.write_u8(*v));
                    bytes_written += 1;
                },
                None => break,
            }
        }
        Ok(bytes_written)
    }

    /// Convert s to a vector with l bytes.
    /// If length of s is > l, the returning vector will only contain the first
    /// l bytes.
    /// Otherwise the returned vector will be filled with \0
    /// until it contains l bytes.
    fn to_bytes(s : &str, l: u32) -> Vec<u8> {
        let mut v = s.to_string().into_bytes();

        v.truncate((l - 1) as usize);

        while v.len() < l as usize {
            v.push(0x00);
        }
        v
    }

}

//---------------------------------------------------------------
// Column
//---------------------------------------------------------------

/// A table column. Has a name, a type, ...
#[derive(Debug,RustcDecodable, RustcEncodable,Clone)]
pub struct Column {
    pub name: String, // name of column
    pub sql_type: SqlType, // name of the data type that is contained in this column
    pub is_primary_key: bool, // defines if column is PK
    pub allow_null: bool, // defines if cloumn allows null
    pub description: String //Displays text describing this column.
}


impl Column {
    /// Creates a new column object
    /// Returns with Column
    pub fn new(
        name: &str,
        sql_type: SqlType,
        allow_null: bool,
        description: &str,
        is_primary_key: bool
        ) -> Column {

        Column {
            name: name.to_string(),
            sql_type: sql_type.clone(),
            allow_null: allow_null,
            description: description.to_string(),
            is_primary_key: is_primary_key
        }
    }

    pub fn get_sql_type(&self) -> &SqlType {
        &self.sql_type
    }
}
