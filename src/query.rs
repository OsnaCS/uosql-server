//! Query excecution moduleT
//!
//! This module takes the output of the SQL parser and executed that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use parse::{ast, parser};
use parse::ast::{Query, CreateStmt, DefStmt, CreateTableStmt};
use super::storage::{Database, Column};


pub struct queryexecutor {
    pub database: Option<Database>,

}


    pub fn execute_from_ast(query: ast::Query, dbase: Option<String>) {


        let mut executor = queryexecutor::new();
         match dbase {
            None => (),
            Some(s) => executor.database =  Database::load(&s).ok(),
        };

        match query {
            Query::ManipulationStmt(stmt) => (),
            Query::DefStmt(stmt) => executor.execute_def_stmt(stmt),
            _ => (),

        }
    //info!("Not implemented! :-(");
    }






impl queryexecutor{

    pub fn new() -> queryexecutor {
        queryexecutor { database: None }
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
                    }).collect();

                    base.create_table(&query.tid, tmp_vec, 0);

                },
            None => (),
        }


    }
}
