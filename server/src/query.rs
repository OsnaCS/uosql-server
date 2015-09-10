//! Query excecution moduleT
//!
//! This module takes the output of the SQL parser and executed that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use super::parse::ast::*;
use super::storage::{Database, Column, Table, Rows};
use super::storage;
use super::auth;
use super::parse::parser::ParseError;
use std::io::{Write, Read, Seek};
use std::fs::File;
use std::io::Cursor;


pub struct Executor<'a> {
    pub user: &'a mut auth::User,
}


    pub fn execute_from_ast<'a, B: Write + Read + Seek>(query: Query, user: &'a mut auth::User)
        -> Result<Rows<B>, ExecutionError> {


        let mut executor = Executor::new(user);

        match query {
            Query::ManipulationStmt(stmt) => executor.execute_manipulation_stmt(stmt),
            Query::DefStmt(stmt) => executor.execute_def_stmt(stmt),
            _ => Err(ExecutionError::ParseError(ParseError::UnknownError)),

        }


    //info!("Not implemented! :-(");
    }




impl<'a> Executor<'a> {


    pub fn new(user: &'a mut auth::User) -> Executor<'a> {
        Executor { user: user }
    }


    fn execute_manipulation_stmt<B: Write + Read + Seek>(&mut self, query: ManipulationStmt)
        -> Result<Rows<B>, ExecutionError> {

        match query {
            ManipulationStmt::Use(stmt) => self.execute_use_stmt(stmt),
            ManipulationStmt::Insert(stmt) => self.execute_insert_stmt(stmt),
            ManipulationStmt::Describe(stmt) => self.execute_describe_stmt(stmt),
            ManipulationStmt::Select(stmt) => self.execute_select_stmt(stmt),
            _ => Err(ExecutionError::DebugError("Feature not implemented yet!".into())),
        }

    }

    fn execute_def_stmt<B: Write + Read + Seek>(&mut self, query: DefStmt) -> Result<Rows<B>, ExecutionError> {
        match query {
            DefStmt::Create(stmt) => self.execute_create_stmt(stmt),
            DefStmt::Drop(stmt) =>  self.execute_drop_stmt(stmt),
            DefStmt::Alter(stmt) => self.execute_alt_stmt(stmt),
        }
    }

    fn execute_use_stmt<B: Write + Read + Seek>(&mut self, query: UseStmt) -> Result<Rows<B>, ExecutionError> {
        match query {
            UseStmt::Database(querybase) => {
                self.user._currentDatabase = Some(try!(Database::load(&querybase)));
                Ok(generate_rows_dummy())
            }

        }
    }

    fn execute_insert_stmt<B: Write + Read + Seek>(&mut self, stmt: InsertStmt) -> Result<Rows<B>, ExecutionError> {
        let table = try!(self.get_table(&stmt.tid));

        if !stmt.col.is_empty() {
            return Err(ExecutionError::DebugError("Not implemented:
            Insert just some values into some columns.
            Use insert into table values (_,....) instead".into()))
        }

        let n: Vec<_> = stmt.val.iter().map(|l| Some(l.into_DataSrc())).collect();
        let mut engine = table.create_engine();
        // try!(engine.insert_row(&n));
        return Err(ExecutionError::DebugError("engine.insert_row() not implemented ".into()));
        // Ok(generate_rows_dummy())

    }

    fn execute_select_stmt<B: Write + Read + Seek>(&mut self, stmt: SelectStmt) -> Result<Rows<B>, ExecutionError> {
        if stmt.target.len() != 1 {
            return Err(ExecutionError::DebugError("Select only implemented for select * ".into()));
        }
        if stmt.target[0].col != Col::Every {
            return Err(ExecutionError::DebugError("Select only implemented for select * ".into()));
        }
        if stmt.tid.len() != 1 {
            return Err(ExecutionError::DebugError("Select only implemented for 1 table ".into()));
        }
        let table = try!(self.get_table(&stmt.tid[0]));
        let engine = table.create_engine();
        // Ok(try!(engine.full_scan()))
        return Err(ExecutionError::DebugError("engine.full_scan() not implemented ".into()));

    }
    fn execute_describe_stmt<B: Write + Read + Seek>(&mut self, query: String) -> Result<Rows<B>, ExecutionError>{
        let table = try!(self.get_table(&query));
        let columns = table.columns();
        let mut columnvec = Vec::new();

        columnvec.extend(columns.iter().cloned());
        //Ok(Rows { data: Vec::new(), columns: columnvec } )
        return Err(ExecutionError::DebugError("Not implemented.".into()));
    }

    fn execute_create_stmt<B: Write + Read + Seek>(&mut self, query: CreateStmt) -> Result<Rows<B>, ExecutionError> {
        match query {
            CreateStmt::Database(s) => {
                self.user._currentDatabase = Some(try!(Database::create(&s)));
                Ok(generate_rows_dummy())
            }
            CreateStmt::Table(stmt) => self.execute_create_table_stmt(stmt),
        }
    }

    fn execute_create_table_stmt<B: Write + Read + Seek>(&mut self, query: CreateTableStmt)
         -> Result<Rows<B>, ExecutionError> {
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
        //Ok(generate_rows_dummy())
        return Err(ExecutionError::DebugError("Not implemented.".into()));
    }

    fn execute_drop_stmt<B: Write + Read + Seek>(&mut self, query: DropStmt) -> Result<Rows<B>, ExecutionError> {
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

    fn execute_alt_stmt<B: Write + Read + Seek>(&mut self, query: AltStmt) -> Result<Rows<B>, ExecutionError> {
        match query {
            AltStmt::Table(stmt) => self.execute_alt_table_stmt(stmt),
        }

    }

    fn execute_alt_table_stmt<B: Write + Read + Seek>(&mut self, stmt: AlterTableStmt) -> Result<Rows<B>, ExecutionError> {
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

    fn get_table(&self, table: &str) -> Result<Table, ExecutionError>{
        let dbase = try!(self.get_own_database());
        Ok(try!(dbase.load_table(table)))
    }


}


fn generate_rows_dummy<B: Write + Read + Seek>() -> Rows<B>{
    /*Rows {
        data: Cursor::<B>::new(Vec::<u8>::new()),
        columns: Vec::new(),
    }*/
    panic!("explicit panic")
}




#[derive(Debug)]
pub enum ExecutionError {

    ParseError(ParseError),
    StorageError(storage::Error),
    UnknownError,
    NoDatabaseSelected,

    DebugError(String),
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
