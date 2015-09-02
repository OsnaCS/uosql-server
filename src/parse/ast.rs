//! Defines types of the abstract syntax tree
//!

/// Top level type. Is returned by `parse`.
#[derive(Debug, Clone)]
pub enum Query {
    ///Data Definition Statement
    DefStmt(DefStmt),
    ///Data Manipulation Statement
    ManipulationStmt
}
#[derive(Debug, Clone)]
pub enum DefStmt {
    ///Create Statement
    Create(CreateStmt),
    ///Alter Statement
    Alter(AltStmt),
    ///Drop Statement
    Drop(DropStmt)
}
#[derive(Debug, Clone)]
pub enum CreateStmt {

    ///Create Table Statement
    Table(CreateTableStmt),

    View
}
#[derive(Debug, Clone)]
pub enum AltStmt {
    //TODO: implement alter statement functionality
    Table,
}
#[derive(Debug, Clone)]
pub enum DropStmt{
    Drop,
}
#[derive(Debug, Clone)]
pub struct CreateTableStmt{
    pub tid: String,
    pub cols: Option<Vec<CreateColumn>>,
}



#[derive(Debug, Clone)]
pub struct CreateColumn {
    pub id: String,
    pub datatype: DType,
}















//general enums
#[derive(Debug, Clone, Copy)]
pub enum DType {
    Int,
    Bool,
    Char(u8),
    VarChar(u16)
}
