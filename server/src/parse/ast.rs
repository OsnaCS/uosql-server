/// Top level type. Is returned by `parse`.
use super::token;
use super::super::storage::SqlType;
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Dummy, // For Compiling
    DefStmt(DefStmt),
    ManipulationStmt(ManipulationStmt)
}

/// All Data Definition Statements
#[derive(Debug, Clone, PartialEq)]
pub enum DefStmt {
    Create(CreateStmt),
    Alter(AltStmt),
    Drop(DropStmt)
}

/// All Data Manipulation Statements
#[derive(Debug, Clone, PartialEq)]
pub enum ManipulationStmt {
    Update(UpdateStmt),
    Select(SelectStmt),
    Insert(InsertStmt),
    Delete(DeleteStmt),
    Use(UseStmt),
    Describe(String),
}

/// Split between creatable content (only Tables yet)
#[derive(Debug, Clone, PartialEq)]
pub enum CreateStmt {
    Table(CreateTableStmt),
    View(CreateViewStmt),
    Database(String),
}

/// Split between alterable content (only Tables yet)
#[derive(Debug, Clone, PartialEq)]
pub enum AltStmt {
    Table(AlterTableStmt)
    //Column(String)
    //View(String)
}

/// Split between drop-able content (only Tables yet)
#[derive(Debug, Clone, PartialEq)]
pub enum DropStmt {
    Table(String),
    View(String),
    Database(String)
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseStmt {
    Database(String)
}

/// Information for table creation
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStmt {
    pub tid: String,
    pub cols: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateViewStmt {
    pub name: String,
    pub opt: bool, // OR REPLACE keyword
    pub sel : SelectStmt,
}

/// Information for column creation
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnInfo {
    pub cid: String,
    pub datatype: SqlType,
    pub primary: bool,
    pub auto_increment: bool,
    pub not_null: bool,
    pub comment: Option<token::Lit>,
}

/// Information for table alteration
#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableStmt {
    pub tid: String,
    pub op: AlterOp
}

/// Possible operations for table alterations
#[derive(Debug, Clone, PartialEq)]
pub enum AlterOp {
    Add(ColumnInfo),
    Drop(String),
    Modify(ColumnInfo)
}

/// Information for table update
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStmt {
    pub tid: String,
    pub alias: HashMap<String, String>,
    pub set: Vec<Condition>,
    pub conds: Option<Conditions>
}

/// Information for data selection
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStmt {
    pub target: Vec<Target>,
    pub tid: Vec<String>,
    pub alias: HashMap<String, String>,
    pub cond: Option<Conditions>,
    //pub groupby: Option<GroupBy>,
    //pub orderby: Option<OrderBy>,
    pub spec_op: Option<SpecOps>,
    pub order: Vec<Sort>,
    pub limit: Option<Limit>,
}

/// Information for data selection
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub alias: Option<String>,
    pub col: Col,
    pub rename: Option<String>,
}

/// Information for data selection in select
#[derive(Debug, Clone, PartialEq)]
pub enum Col {
    // select a specified column
    Specified(String),
    // for example: table.* => select every column in table
    Every

}

/// Information for data output limiting
#[derive(Debug, Clone, PartialEq)]
pub struct Limit {
    //limit the count of the output
    pub count: Option<i64>,
    //offset the output: 0 = no offset, n = display from the nth row
    pub offset: Option<i64>,
}

/// Information for data insertion
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStmt {
    pub tid: String,
    pub col: Vec<String>,
    pub val: Vec<token::Lit>
}

/// Information for data deletion
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStmt {
    pub tid: String,
    pub alias: HashMap<String, String>,
    pub cond: Option<Conditions>
}

/// Additional operations for ordering and limiting
#[derive(Debug, Clone, PartialEq)]
pub enum SpecOps {
    OrderByAsc(String),
    OrderByDesc(String),
    GroupBy(Vec<String>),
    Limit(u32)
}

/// Conditions for managing AND/OR where-clauses
#[derive(Debug, Clone, PartialEq)]
pub enum Conditions {
    Leaf(Condition),
    And(Box<Conditions>, Box<Conditions>),
    Or(Box<Conditions>, Box<Conditions>)
}

/// Information for the where-clause
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub aliascol: Option<String>,
    pub col: String,
    pub op: CompType,
    // in where clause, the condition may consist of two column names,
    // this alaiasrhs is existent, if the right side is a word (=column)
    // and if there exists an alias in the sql statement
    // example: where p.name = s.name
    pub aliasrhs: Option<String>,
    pub rhs: CondType
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sort {
    pub alias: Option<String>,
    pub col: String,
    pub order: Option<Order>,
}

/// Allowed operators for where-clause
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CompType {
    Equ,
    NEqu,
    GThan,
    SThan,
    GEThan,
    SEThan
}

/// Allowed data types for where-clause
#[derive(Debug, Clone, PartialEq)]
pub enum CondType {
    Literal(token::Lit),
    Word(String)
}

#[derive(Debug, PartialEq)]
pub enum DataSrc {
    Int(i64),
    String(String),
    Bool(u8),
}

/// Possible values for "Order By" keyword
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Asc,
    Desc
}
