use super::{Error};

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
            &SqlType::Char(len) => (len + 1) as u32,
            &SqlType::VarChar(len) => (len + 1) as u32
        }
    }

    encode_into<W: Write>(&self, buf: W, data: ast::DataSrc) -> Result<u32, Error> {

    }
}
