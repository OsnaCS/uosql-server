//! Query excecution moduleT
//!
//! This module takes the output of the SQL parser and executed that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use super::parse::ast::*;
use super::storage::{Database, Column, Table, Rows, ResultSet, Engine,EngineID};
use super::storage;
use super::auth;
use super::parse::parser::ParseError;
use std::io::{Write, Read, Seek};
use std::fs::File;
use std::io::Cursor;
use std::collections::HashMap;

pub struct Executor<'a> {
    pub user: &'a mut auth::User,
}



    pub fn execute_from_ast<'a>(query: Query, user: &'a mut auth::User)
        -> Result<ResultSet, ExecutionError> {

        let mut executor = Executor::new(user);

        let res = match query {
            Query::ManipulationStmt(stmt) => executor.execute_manipulation_stmt(stmt),
            Query::DefStmt(stmt) => executor.execute_def_stmt(stmt),
            _ => return Err(ExecutionError::ParseError(ParseError::UnknownError)),

        };
        Ok(try!(try!(res).to_result_set()))
    }




impl<'a> Executor<'a> {


    pub fn new(user: &'a mut auth::User) -> Executor<'a> {
        Executor { user: user }
    }


    fn execute_manipulation_stmt(&mut self, query: ManipulationStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError> {


        match query {
            ManipulationStmt::Use(stmt) => self.execute_use_stmt(stmt),
            ManipulationStmt::Insert(stmt) => self.execute_insert_stmt(stmt),
            ManipulationStmt::Describe(stmt) => self.execute_describe_stmt(stmt),
            ManipulationStmt::Select(stmt) => self.execute_select_stmt(stmt),
            _ => Err(ExecutionError::DebugError("Feature not implemented yet!".into())),
        }

    }

    fn execute_def_stmt(&mut self, query: DefStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {

        match query {
            DefStmt::Create(stmt) => self.execute_create_stmt(stmt),
            DefStmt::Drop(stmt) =>  self.execute_drop_stmt(stmt),
            DefStmt::Alter(stmt) => self.execute_alt_stmt(stmt),
        }
    }

    fn execute_use_stmt(&mut self, query: UseStmt)
    -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {

        match query {
            UseStmt::Database(querybase) => {
                self.user._currentDatabase = Some(try!(Database::load(&querybase)));
                Ok(generate_rows_dummy())
            }
        }
    }

    fn execute_insert_stmt(&mut self, stmt: InsertStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        let table = try!(self.get_table(&stmt.tid));

        if !stmt.col.is_empty() {
            return Err(ExecutionError::DebugError("Not implemented:
            Insert just some values into some columns.
            Use insert into table values (_,....) instead".into()))
        }

        let mut writevec = Vec::<u8>::new();
        {
            let columns = table.columns();
            let insertvalues = stmt.val;
            if insertvalues.len() != columns.len() {
                return Err(ExecutionError::InsertMissmatch)
            }

            let mut index = 0;

            for column in table.columns() {
                info!("inserting at {:?}", writevec.len());
                info!("This is the insertvalue: {:?}", insertvalues[index] );
                column.sql_type.encode_into(&mut writevec,&insertvalues[index]);
                index += 1;
            }
        }
        let mut engine = table.create_engine();
        info!("handing data vector {:?} to storage engine",writevec);
        try!(engine.insert_row(&writevec));
        Ok(generate_rows_dummy())

    }

fn execute_select_stmt(&mut self, stmt: SelectStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        let mut masterrow: Rows<Cursor<Vec<u8>>>;

        let mut left = try!(self.get_rows(&stmt.tid[0]));
        println!("{:?}", left );
        let mut name_column_map = HashMap::<String, HashMap<String, usize>>::new();
        let mut column_index_map = HashMap::<String, usize>::new();
        let mut columnindex: usize = 0;
        for column in left.columns.clone() {
            column_index_map.insert(column.name.into(), columnindex);
            columnindex += 1;
        }
        name_column_map.insert(stmt.tid[0].clone(), column_index_map);

        // create a very huge cross product from all tables and some hashmaputilities
        for i in 1..stmt.tid.len() {

            let right = try!(self.get_rows(&stmt.tid[i]));
            column_index_map = HashMap::<String, usize>::new();
            for column in right.columns.clone() {
                column_index_map.insert(column.name.into(), columnindex);
                columnindex += 1;
            }
            name_column_map.insert(stmt.tid[i].clone(), column_index_map);

            let tmp = self.cross_rows(left, right);
            left = tmp;


        }
        masterrow = left;

        println!("{:?}",name_column_map);
        // create the hashmap that gives every alias it's tablehash where the tablehash
        // maps a columnname to a index

        for i in 0..stmt.tid.len() {

        }



        let result = if stmt.cond.is_some() {
                try!(self.execute_where(masterrow, &stmt.alias, &stmt.cond.unwrap()))
            } else {
                masterrow
            };
        Ok(result)

        /*if stmt.target.len() != 1 {
            return Err(ExecutionError::DebugError("Select only implemented for select * ".into()));
        }
        if stmt.target[0].col != Col::Every {
            return Err(ExecutionError::DebugError("Select only implemented for select * ".into()));
        }
        if stmt.tid.len() != 1 {
            return Err(ExecutionError::DebugError("Select only implemented for 1 table ".into()));
        }
        let engine = try!(self.get_engine(&stmt.tid[0]));
        Ok(try!(engine.full_scan()))*/

    }

    fn execute_where<'b>(&self, tableset:Rows<Cursor<Vec<u8>>>
        , infos: &HashMap<String, String>, conditions: &Conditions)
            -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError> {

                match conditions {
                    &Conditions::And(ref c1, ref c2) => {
                        Ok(tableset)
                        },
                    &Conditions::Or(ref c1, ref c2) => Ok(tableset),
                    &Conditions::Leaf(ref c) => {
                        Ok(tableset)

                    },
                }

    }

    fn execute_describe_stmt(&mut self, query: String)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        let table = try!(self.get_table(&query));
        let columns = table.columns();
        let mut columnvec = Vec::new();

        columnvec.extend(columns.iter().cloned());
        Ok(Rows::new(Cursor::new(Vec::<u8>::new()), &columnvec))
    }

    fn execute_create_stmt(&mut self, query: CreateStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        match query {
            CreateStmt::Database(s) => {
                self.user._currentDatabase = Some(try!(Database::create(&s)));
                Ok(generate_rows_dummy())
            },
            CreateStmt::Table(stmt) => self.execute_create_table_stmt(stmt),
            _ => Err(ExecutionError::DebugError("to_do".into())),
        }
    }

    fn execute_create_table_stmt(&mut self, query: CreateTableStmt)
         -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError> {
        let base = try!(self.get_own_database());
        let tmp_vec : Vec<_> = query.cols.into_iter().map(|c| Column {
            name: c.cid,
            sql_type: c.datatype,
            allow_null: false,
            description: "this is a column".to_string(),
             is_primary_key: c.primary,
        }).collect();
        let table = try!(base.create_table(&query.tid, tmp_vec, EngineID::FlatFile));
        let mut engine = table.create_engine();
        engine.create_table();
        Ok(generate_rows_dummy())
    }

    fn execute_drop_stmt(&mut self, query: DropStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
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
            _ => Err(ExecutionError::DebugError("to_do".into())),
        }
    }


    fn execute_alt_stmt(&mut self, query: AltStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        match query {
            AltStmt::Table(stmt) => self.execute_alt_table_stmt(stmt),
        }

    }


    fn execute_alt_table_stmt(&mut self, stmt: AlterTableStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
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

    fn get_engine<'b>(&'b self, table: &str) -> Result<Box<Engine + 'b>, ExecutionError> {
        let table = try!(self.get_table(table));
        Ok(table.create_engine())
    }

    fn get_rows(&self, table: &str) -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError> {
        let engine = try!(self.get_engine(table));
        Ok(try!(engine.full_scan()))
    }

    fn cross_rows(&self, mut left: Rows<Cursor<Vec<u8>>>, mut right: Rows<Cursor<Vec<u8>>>)
        -> Rows<Cursor<Vec<u8>>>
    {
        let mut datasrc = Vec::<u8>::new();

        loop {
            let outerres = left.next_row(&mut datasrc);
            match outerres {
                Ok(_) => (),
                Err(_) => break
            }
                loop {
                    let innerres = right.next_row(&mut datasrc);
                    match innerres {
                    Ok(_) => (),
                    Err(_) => break
                    }
                }

        }

        let mut columnvec = left.columns;
        let mut appendvec = right.columns;

        for i in 0..appendvec.len() {
            let appendlength = appendvec.len();
            columnvec.push(appendvec.remove(0));
        }

        let mut cursor = Cursor::new(datasrc);

        Rows::<Cursor<Vec<u8>>>::new(cursor, &columnvec)

    }



}


fn generate_rows_dummy() -> Rows<Cursor<Vec<u8>>> {
    let v = Vec::<u8>::new();
    let c = Cursor::new(v);

    Rows::new(c, &[])
}




#[derive(Debug)]
pub enum ExecutionError {

    ParseError(ParseError),
    StorageError(storage::Error),
    UnknownError,
    NoDatabaseSelected,
    InsertMissmatch,
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
