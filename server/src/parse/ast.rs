/// Top level type. Is returned by `parse`.
use super::token;
use super::super::storage::SqlType;
#[derive(Debug, Clone)]
pub enum Query {
    Dummy, // For Compiling
    DefStmt(DefStmt),
    ManipulationStmt(ManipulationStmt)
}

/// All Data Definition Statements
#[derive(Debug, Clone)]
pub enum DefStmt {
    Create(CreateStmt),
    Alter(AltStmt),
    Drop(DropStmt)
}

/// All Data Manipulation Statements
#[derive(Debug, Clone)]
pub enum ManipulationStmt {
    Update(UpdateStmt),
    Select(SelectStmt),
    Insert(InsertStmt),
    Delete(DeleteStmt),
    Use(UseStmt)
}

/// Split between creatable content (only Tables yet)
#[derive(Debug, Clone)]
pub enum CreateStmt {
    Table(CreateTableStmt),
    // View
    Database(String),
}

/// Split between alterable content (only Tables yet)
#[derive(Debug, Clone)]
pub enum AltStmt {
    Table(AlterTableStmt)
    //Column(String)
    //View(String)
}

/// Split between drop-able content (only Tables yet)
#[derive(Debug, Clone)]
pub enum DropStmt {
    Table(String),
    //Index(String)
    Database(String)
}

#[derive(Debug, Clone)]
pub enum UseStmt {
    Database(String)
}

/// Information for table creation
#[derive(Debug, Clone)]
pub struct CreateTableStmt {
    pub tid: String,
    pub cols: Vec<ColumnInfo>,
}

/// Information for column creation
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub cid: String,
    pub datatype: SqlType,
}

/// Information for table alteration
#[derive(Debug, Clone)]
pub struct AlterTableStmt {
    pub tid: String,
    pub op: AlterOp
}

/// Possible operations for table alterations
#[derive(Debug, Clone)]
pub enum AlterOp {
    Add(ColumnInfo),
    Drop(String),
    Modify(ColumnInfo)
}

/// Information for table update
#[derive(Debug, Clone)]
pub struct UpdateStmt {
    pub tid: String,
    pub set: Vec<Condition>,
    pub conds: Option<Conditions>
}

/// Information for data selection
#[derive(Debug, Clone)]
pub struct SelectStmt {
    pub target: Vec<String>,
    pub tid: Vec<String>,
    pub cond: Option<Conditions>,
    pub spec_op: Option<SpecOps>
}

/// Information for data insertion
#[derive(Debug, Clone)]
pub struct InsertStmt {
    pub tid: String,
    pub col: Vec<String>,
    pub val: Vec<token::Lit>
}

/// Information for data deletion
#[derive(Debug, Clone)]
pub struct DeleteStmt {
    pub tid: String,
    pub cond: Option<Conditions>
}

/// Additional operations for ordering and limiting
#[derive(Debug,Clone)]
pub enum SpecOps {
    OrderByAsc(String),
    OrderByDesc(String),
    GroupBy(Vec<String>),
    Limit(u32)
}

/// Conditions for managing AND/OR where-clauses
#[derive(Debug, Clone)]
pub enum Conditions {
    Leaf(Condition),
    And(Box<Conditions>, Box<Conditions>),
    Or(Box<Conditions>, Box<Conditions>)
}

/// Information for the where-clause
#[derive(Debug, Clone)]
pub struct Condition {
    pub col: String,
    pub op: CompType,
    pub rhs: CondType
}

/// Allowed operators for where-clause
#[derive(Debug, Clone, Copy)]
pub enum CompType {
    Equ,
    NEqu,
    GThan,
    SThan,
    GEThan,
    SEThan
}

/// Allowed data types for where-clause
#[derive(Debug, Clone)]
pub enum CondType {
    Literal(String),
    Num(f32),
    Word(String)
}

pub enum DataSrc  {
    Int(i64),
    String(String),
    Bool(u8),

}
