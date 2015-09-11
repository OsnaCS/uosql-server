//! Query excecution moduleT
//!
//! This module takes the output of the SQL parser and executed that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use super::parse::ast::*;
use super::storage::{Database, Column, Table, Rows, Engine};
use super::storage;
use super::auth;
use super::parse::parser::ParseError;
use std::collections::HashMap;


pub struct Executor<'a> {
    pub user: &'a mut auth::User,
}


    pub fn execute_from_ast<'a>(query: Query, user: &'a mut auth::User)
        -> Result<Rows, ExecutionError> {

        let mut executor = Executor::new(user);
        match query {
            Query::ManipulationStmt(stmt) => executor.execute_manipulation_stmt(stmt),
            Query::DefStmt(stmt) => executor.execute_def_stmt(stmt),
            _ => Err(ExecutionError::ParseError(ParseError::UnknownError)),
        }
    }




impl<'a> Executor<'a> {


    pub fn new(user: &'a mut auth::User) -> Executor<'a> {
        Executor { user: user }
    }


    fn execute_manipulation_stmt(&mut self, query: ManipulationStmt)
        -> Result<Rows, ExecutionError> {

        match query {
            ManipulationStmt::Use(stmt) => self.execute_use_stmt(stmt),
            ManipulationStmt::Describe(stmt) => self.execute_describe_stmt(stmt),
            ManipulationStmt::Insert(stmt) => self.execute_insert_stmt(stmt),
            ManipulationStmt::Update(stmt) => Ok(generate_rows_dummy()),
            ManipulationStmt::Delete(stmt) => Ok(generate_rows_dummy()),
            ManipulationStmt::Select(stmt) => self.execute_select_stmt(stmt),
        }

    }

    fn execute_def_stmt(&mut self, query: DefStmt) -> Result<Rows, ExecutionError> {
        match query {
            DefStmt::Create(stmt) => self.execute_create_stmt(stmt),
            DefStmt::Drop(stmt) =>  self.execute_drop_stmt(stmt),
            DefStmt::Alter(stmt) => self.execute_alt_stmt(stmt),
        }
    }

    fn execute_use_stmt(&mut self, query: UseStmt) -> Result<Rows, ExecutionError> {
        match query {
            UseStmt::Database(querybase) => {
                self.user._currentDatabase = Some(try!(Database::load(&querybase)));
                Ok(generate_rows_dummy())
            },

        }
    }


    fn execute_describe_stmt(&mut self, query: String) -> Result<Rows, ExecutionError>{
        let table = try!(self.get_table(&query));
        let columns = table.columns();
        let mut columnvec = Vec::new();

        columnvec.extend(columns.iter().cloned());
        Ok(Rows { data: Vec::new(), columns: columnvec } )
    }

    fn execute_insert_stmt(&mut self, stmt: InsertStmt) -> Result<Rows, ExecutionError> {
        let table = try!(self.get_table(&stmt.tid));

        if !stmt.col.is_empty() {
            return Err(ExecutionError::DebugError("Not implemented:
            Insert just some values into some columns.
            Use insert into table values (_,....) instead".into()))
        }

        let n: Vec<_> = stmt.val.iter().map(|l| Some(l.into_DataSrc())).collect();
        let mut engine = table.create_engine();
        try!(engine.insert_row(&n));
        Ok(generate_rows_dummy())

    }

    fn execute_update_stmt(&mut self, stmt: UpdateStmt) -> Result <Rows, ExecutionError> {

        Ok(generate_rows_dummy())
    }

    fn execute_delete_stmt(&mut self, stmt: DeleteStmt) -> Result <Rows, ExecutionError> {
        Ok(generate_rows_dummy())
    }


    fn execute_select_stmt(&mut self, stmt: SelectStmt) -> Result<Rows, ExecutionError> {
        let col2tbl: HashMap<String, String> = HashMap::new();
        let mut tablemap = HashMap::new();

        for tablestring in stmt.tid.clone() {
            let  table = try!(self.get_table(&tablestring));
            for column in table.columns().clone() {

            }
            let rows = try!(table.create_engine().full_scan());

            tablemap.insert(tablestring, rows);
        }

        if stmt.cond.is_some() {
            let result = try!(self.execute_where(& mut tablemap, &stmt.alias, &stmt.cond.unwrap()));
        } else {

        }



        if stmt.target.len() != 1 {
            return Err(ExecutionError::DebugError("Select only implemented for select * ".into()));
        }
        if stmt.target[0].col != Col::Every {
            return Err(ExecutionError::DebugError("Select only implemented for select * ".into()));
        }
        if stmt.tid.len() != 1 {
            return Err(ExecutionError::DebugError("Select only implemented for 1 table ".into()));
        }
        let engine = try!(self.get_engine(&stmt.tid[0]));
        Ok(try!(engine.full_scan()))

    }

    fn execute_where<'b>(&self, tableset: &'b mut HashMap<String,
        Rows> ,infos: &HashMap<String, String>, conditions: &Conditions)
            -> Result<&'b mut HashMap<String, Rows>, ExecutionError> {

                match conditions {
                    &Conditions::And(ref c1, ref c2) => {
                        let set1 = try!(self.execute_where(tableset, infos, c1));
                        Ok(try!(self.execute_where(set1, infos, c2)))
                        },
                    &Conditions::Or(ref c1, ref c2) => Ok(tableset),
                    &Conditions::Leaf(ref c) => {
                        let mut table;

                        match c.aliascol {
                            Some(ref c) => table = infos.get(c).unwrap(),
                            None => return Err(ExecutionError::DebugError(
                                "Use alias in selections! (or given table not found)".into()))
                        }


                        {
                        let rows = match tableset.get_mut(table) {
                            Some(tableset2) => tableset2,
                            None => return Err(ExecutionError::UnknownError)
                        };
                            let iterator = rows.iter();
                            for row in iterator {
                                let mut index = 0;
                                let mut valid = false;
                                for column in row.owner.columns.clone() {
                                    if (c.col == column.name) {
                                        valid = true;
                                        break
                                    }
                                    index +=1;
                                }
                                if !valid {
                                    return Err(ExecutionError::NoSuchColumn(c.col.clone()))
                                }

                                let stype = row.owner.columns[index].sql_type;
                                let comp = match &c.rhs {
                                    &CondType::Word(_) => return Err(ExecutionError::DebugError("Not implemented yet!".to_string())),
                                    &CondType::Literal(ref lit) => lit.into_DataSrc(),
                                };

                                //storage::SqlType::Int.decode_from()

                            }
                        }
                        Ok(tableset)

                    },
                }


    }

    fn execute_create_stmt(&mut self, query: CreateStmt) -> Result<Rows, ExecutionError> {
        match query {
            CreateStmt::Database(s) => {
                self.user._currentDatabase = Some(try!(Database::create(&s)));
                Ok(generate_rows_dummy())
            }
            CreateStmt::Table(stmt) => self.execute_create_table_stmt(stmt),
        }
    }

    fn execute_create_table_stmt(&mut self, query: CreateTableStmt)
         -> Result<Rows, ExecutionError> {
        let base = try!(self.get_own_database());
        let tmp_vec : Vec<_> = query.cols.into_iter().map(|c| Column {
            name: c.cid,
            sql_type: c.datatype,
            allow_null: false,
            description: "this is a column".to_string(),
             is_primary_key: c.primary,
        }).collect();
        let table = try!(base.create_table(&query.tid, tmp_vec, 0));
        let mut engine = table.create_engine();
        engine.create_table();
        Ok(generate_rows_dummy())
    }

    fn execute_drop_stmt(&mut self, query: DropStmt) -> Result<Rows, ExecutionError> {
        match query {
            DropStmt::Table(s) => {
                let base = try!(self.get_own_database());
                let table = try!(base.load_table(&s));
                try!(table.delete());
                Ok(generate_rows_dummy())
            },
            DropStmt::Database(s) => {
                let base = try!(Database::load(&s));
                try!(base.delete());
                let mut baseinuse = false;
                match self.user._currentDatabase {
                    Some(ref base) => {
                        if base.name == s {
                            baseinuse = true;
                        };
                        ()
                    },
                    None => (),
                };
                if baseinuse {
                    self.user._currentDatabase = None;
                };
                Ok(generate_rows_dummy())
            },
        }
    }

    fn execute_alt_stmt(&mut self, query: AltStmt) -> Result<Rows, ExecutionError> {
        match query {
            AltStmt::Table(stmt) => self.execute_alt_table_stmt(stmt),
        }

    }

    fn execute_alt_table_stmt(&mut self, stmt: AlterTableStmt) -> Result<Rows, ExecutionError> {
        let table = try!(self.get_table(&stmt.tid));
        match stmt.op {
            AlterOp::Add(columninfo) => Ok(generate_rows_dummy()),
            AlterOp::Drop(column) => Ok(generate_rows_dummy()),
            AlterOp::Modify(columninfo) => Ok(generate_rows_dummy()),
        }


    }






    fn get_own_database(&self) -> Result<&Database, ExecutionError> {
        match self.user._currentDatabase {
            Some(ref base) => Ok(base),
            None => Err(ExecutionError::NoDatabaseSelected),
        }
    }

    fn get_table(&self, table: &str) -> Result<Table, ExecutionError> {
        let dbase = try!(self.get_own_database());
        Ok(try!(dbase.load_table(table)))
    }

    fn get_engine<'b>(&'b self, table: &str) -> Result<Box<Engine + 'b>, ExecutionError> {
        let table = try!(self.get_table(table));
        Ok(table.create_engine())
    }

    fn get_rows(&self, table: &str) -> Result<Rows, ExecutionError> {
        let engine = try!(self.get_engine(table));
        Ok(try!(engine.full_scan()))
    }
}


fn generate_rows_dummy() -> Rows {
    Rows {
    data: Vec::new(),
    columns: Vec::new(),
}
}




#[derive(Debug)]
pub enum ExecutionError {

    ParseError(ParseError),
    StorageError(storage::Error),
    UnknownError,
    NoDatabaseSelected,
    NoSuchAlias(String),
    DebugError(String),
    NoSuchColumn(String),
}

impl From<ParseError> for ExecutionError {
    fn from(error: ParseError) -> ExecutionError {
        ExecutionError::ParseError(error)
    }
}

impl From<storage::Error> for ExecutionError {
    fn from(error: storage::Error) -> ExecutionError {
        ExecutionError::StorageError(error)
    }
}
