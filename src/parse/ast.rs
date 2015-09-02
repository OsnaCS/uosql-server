/// Top level type. Is returned by `parse`.
#[derive(Debug, Clone)]
pub enum Query {
	// Select,
    DefStmt(DefStmt),
    ManipulationStmt(ManipulationStmt)
}

#[derive(Debug, Clone)]
pub enum DefStmt {
    Create(CreateStmt),
    Alter(AltStmt),
    Drop(DropStmt)
}

#[derive(Debug, Clone)]
pub enum ManipulationStmt {
	Update(UpdateStmt),
	Select(SelectStmt),
	Insert(InsertStmt),
	Delete(DeleteStmt)
}

#[derive(Debug, Clone)]
pub enum CreateStmt {
    Table(CreateTableStmt),
    // View
}

#[derive(Debug, Clone)]
pub enum AltStmt {
    Table(AlterTableStmt)
    //Column(String)
    //View(String)
}

#[derive(Debug, Clone)]
pub enum DropStmt {
    Table(String)
    //Column(String)
    //Index(String)
    //Database(String)
}

#[derive(Debug, Clone)]
pub struct CreateTableStmt {
    pub tid: String,
    pub cols: Option<Vec<CreateColumn>>,
}

#[derive(Debug, Clone)]
pub struct CreateColumn {
    pub id: String,
    pub datatype: DType,
}

#[derive(Debug, Clone)]
pub struct AlterTableStmt {
	pub id: String,
	pub op: AlterOp
}

#[derive(Debug, Clone)]
pub enum AlterOp {
	Add(CreateColumn),
	Drop(String),
	Alter(CreateColumn)
}

#[derive(Debug, Clone)]
pub struct UpdateStmt {
	pub tid: String,
	pub set: Vec<Condition>,
	pub conds: Option<Conditions>
}

#[derive(Debug, Clone)]
pub struct SelectStmt {
	pub target: Vec<String>,
	pub tid: Vec<String>,
	pub cond: Option<Conditions>,
	pub spec_op: Option<SpecOps>
}

#[derive(Debug, Clone)]
pub struct InsertStmt {
	pub tid: String,
	pub col: Option<Vec<String>>,
	pub val: Vec<DType>
}

#[derive(Debug, Clone)]
pub struct DeleteStmt {
	pub tid: String,
	pub cond: Option<Conditions>
}

#[derive(Debug,Clone)]
pub enum SpecOps {
	OrderByAsc(String),
	OrderByDesc(String),
	GroupBy(Vec<String>),
	Limit(u32)
}

#[derive(Debug, Clone)]
pub enum Conditions {
	// Leaf
	Leaf(Condition),
	And((Box<Conditions>, Box<Conditions>)),
	Or((Box<Conditions>, Box<Conditions>))
}

#[derive(Debug, Clone)]
pub struct Condition {
	pub a : CondType,
	pub op: CompType,
	pub b : CondType
}

#[derive(Debug, Clone)]
pub enum CompType {
	Equ,
	NEqu,
	GThan,
	SThan,
	GEThan,
	SEThan
}

#[derive(Debug, Clone)]
pub enum CondType {
	Literal(String),
	Num(f32),
	Word(String)
}

//general enums
#[derive(Debug, Clone, Copy)]
pub enum DType {
    Int,
    Bool,
    Char(u8),
    VarChar(u16)
}
