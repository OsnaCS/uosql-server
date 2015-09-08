use super::{Error};
use std::io::Write;
use std::io::Read;
use super::super::parse::ast::DataSrc;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};


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

    /// Decodes the data in buf according to SqlType into a DataSrc enum.
    pub fn decode_from<R: Read>(&self, mut buf: &mut R) -> Result<DataSrc, Error> {
        match self {
            &SqlType::Int => {
                let i = try!(buf.read_i32::<BigEndian>());
                Ok(DataSrc::Int(i as i64))
            },
            &SqlType::Bool => {
                let b = try!(buf.read_u8());
                Ok(DataSrc::Bool(b))
            },
            &SqlType::Char(_) => {
                let mut s = String::new();
                try!(buf.read_to_string(&mut s));
                Ok(DataSrc::String(s))
            },
            &SqlType::VarChar(_) => {
                let mut s = String::new();
                try!(buf.read_to_string(&mut s));
                Ok(DataSrc::String(s))
            }
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
