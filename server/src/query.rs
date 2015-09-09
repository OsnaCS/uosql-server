//! Query excecution moduleT
//!
//! This module takes the output of the SQL parser and executed that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use super::parse::ast;
use super::parse::ast::{Query, CreateStmt, DefStmt, CreateTableStmt};
use super::storage::{Database, Column};


pub struct Executor {
    pub database: Option<Database>,
}


    pub fn execute_from_ast(query: ast::Query, dbase: Option<String>) {


        let mut executor = Executor::new();
         match dbase {
            None => (),
            Some(s) => executor.database =  Database::load(&s).ok(),
        };

        match query {
            Query::ManipulationStmt(_) => (),
            Query::DefStmt(stmt) => executor.execute_def_stmt(stmt),
            _ => (),

        }
    //info!("Not implemented! :-(");
    }


impl Executor{

    pub fn new() -> Executor {
        Executor { database: None }
    }



    fn execute_def_stmt(&mut self, query: ast::DefStmt) {
        match query {
            DefStmt::Create(stmt) => self.execute_create_stmt(stmt),
            _ => (),
            //Drop
            //Alter

        }

    }

    fn execute_create_stmt(&mut self, query: ast::CreateStmt) {
        match query {

            CreateStmt::Database(s) =>{
                let tmp = Database::create(&s);
                match tmp {
                    Ok(dbase) => self.database = Some(dbase),
                    _ => (),
                }
            }

            CreateStmt::Table(stmt) => self.execute_create_table_stmt(stmt),
        }

    }

    fn execute_create_table_stmt(&mut self, query: CreateTableStmt) {

        match self.database {
            Some(ref base) => {


                let tmp_vec : Vec<_> = query.cols.into_iter().map(|c| Column {
                    name: c.cid,
                    sql_type: c.datatype,
                    allow_null: false,
                    description: "this is a column".to_string(),
                    is_primary_key: c.primary,
                }).collect();

                // TODO: Use the result!
                let _ = base.create_table(&query.tid, tmp_vec, 0);

            },
            None => (),
        }


    }
}
