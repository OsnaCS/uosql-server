//! Query excecution moduleT
//!
//! This module takes the output of the SQL parser and executed that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use super::parse::ast::*;
use super::storage::{Database, Column, Table};
use super::storage;
use super::auth;
use super::parse::parser::ParseError;


pub struct Executor<'a> {
    pub user: &'a mut auth::User,
}


    pub fn execute_from_ast<'a>(query: Query, user: &'a mut auth::User) -> Result<(), ExecutionError> {


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


    fn execute_manipulation_stmt(&mut self, query: ManipulationStmt) -> Result<(), ExecutionError> {

        match query {
            ManipulationStmt::Use(stmt) => self.execute_use_stmt(stmt),
            ManipulationStmt::Insert(stmt) => self.execute_insert_stmt(stmt),
            _ => Ok(()),
        }

    }

    fn execute_use_stmt(&mut self, query: UseStmt) -> Result<(), ExecutionError> {
        match query {
            UseStmt::Database(querybase) => {
                self.user._currentDatabase = Some(try!(Database::load(&querybase)));
                Ok(())
            }

        }
    }

    fn execute_def_stmt(&mut self, query: DefStmt) -> Result<(), ExecutionError> {
        match query {
            DefStmt::Create(stmt) => self.execute_create_stmt(stmt),
            DefStmt::Drop(stmt) =>  self.execute_drop_stmt(stmt),
            DefStmt::Alter(stmt) => self.execute_alt_stmt(stmt),
        }
    }

    fn execute_create_stmt(&mut self, query: CreateStmt) -> Result<(), ExecutionError> {
        match query {
            CreateStmt::Database(s) => {
                self.user._currentDatabase = Some(try!(Database::create(&s)));
                Ok(())
            }
            CreateStmt::Table(stmt) => self.execute_create_table_stmt(stmt),
        }
    }

    fn execute_create_table_stmt(&mut self, query: CreateTableStmt) -> Result<(), ExecutionError> {
        let base = try!(self.get_own_database());
        let tmp_vec : Vec<_> = query.cols.into_iter().map(|c| Column {
            name: c.cid,
            sql_type: c.datatype,
            allow_null: false,
            description: "this is a column".to_string(),
             is_primary_key: c.primary,
        }).collect();
        try!(base.create_table(&query.tid, tmp_vec, 0));
        Ok(())
    }

    fn execute_drop_stmt(&mut self, query: DropStmt) -> Result<(), ExecutionError> {
        match query {
            DropStmt::Table(s) => {
                let base = try!(self.get_own_database());
                let table = try!(base.load_table(&s));
                Ok(try!(table.delete()))
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
                Ok(())
            },
        }
    }

    fn execute_alt_stmt(&mut self, query: AltStmt) -> Result<(), ExecutionError> {
        match query {
            AltStmt::Table(stmt) => self.execute_alt_table_stmt(stmt),
        }

    }

    fn execute_alt_table_stmt(&mut self, stmt: AlterTableStmt) -> Result<(), ExecutionError> {
        let table = try!(self.get_table(&stmt.tid));
        match stmt.op {
            AlterOp::Add(columninfo) => Ok(()),
            AlterOp::Drop(column) => Ok(()),
            AlterOp::Modify(columninfo) => Ok(()),
        }


    }

    fn execute_insert_stmt(&mut self, stmt: InsertStmt) -> Result<(), ExecutionError> {
        let table = self.get_table(&stmt.tid);
        if !stmt.col.is_empty() {
            return Err(ExecutionError::DebugError("Not implemented:
            Insert just some values into some columns.
            Use insert into table values (_,....) instead".into()))
        }
        
        Ok(())

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
