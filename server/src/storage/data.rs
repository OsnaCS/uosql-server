use std::vec::Vec;
use super::Table;
use super::Error;
use super::super::parse::ast::DataSrc;
use super::types::SqlType;
use super::types::Column;
use super::types::FromSql;
use byteorder::{BigEndian, ReadBytesExt};


#[derive(Debug, Default)]
pub struct Rows {
    pub data: Vec<u8>,
    pub columns: Vec<Column>,
}


/// Represents the lines read from file.
impl Rows {

}
