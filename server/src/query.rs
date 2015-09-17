//! Query excecution moduleT
//!
//! This module takes the output of the SQL parser and executed that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use super::parse::ast::*;
use super::parse::token::Lit;
use super::storage::{Database, Column, Table, Rows, ResultSet, Engine, EngineID, Error};
use super::storage::types::SqlType;
use super::storage;
use super::auth;
use super::parse::parser::ParseError;
use std::io::{Write, Read, Seek};
use std::fs::File;
use std::io::Cursor;
use std::collections::HashMap;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

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


    fn execute_manipulation_stmt(&mut self, mut query: ManipulationStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError> {


        match query {
            ManipulationStmt::Use(stmt) => self.execute_use_stmt(stmt),
            ManipulationStmt::Insert(stmt) => self.execute_insert_stmt(stmt),
            ManipulationStmt::Describe(stmt) => self.execute_describe_stmt(stmt),
            ManipulationStmt::Select(stmt) => self.execute_select_stmt(stmt),
            ManipulationStmt::Delete(stmt) => self.execute_delete_stmt(stmt),
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

fn execute_select_stmt(&mut self, mut stmt: SelectStmt)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        let mut masterrow: Rows<Cursor<Vec<u8>>>;

        let mut left = try!(self.get_rows(&stmt.tid[0]));

        let mut name_column_map = HashMap::<String, HashMap<String, usize>>::new();
        let mut column_index_map = HashMap::<String, usize>::new();
        let mut column_tablename_map = HashMap::<String, String>::new();
        let mut columnindex: usize = 0;
        for column in left.columns.clone() {
            column_tablename_map.insert(column.name.clone(), stmt.tid[0].clone());
            column_index_map.insert(column.name.into(), columnindex);
            columnindex += 1;
        }
        name_column_map.insert(stmt.tid[0].clone(), column_index_map);
        stmt.alias.insert(stmt.tid[0].clone(), stmt.tid[0].clone());


        // create a very huge cross product from all tables and some hashmaputilities
        for i in 1..stmt.tid.len() {
            let right = try!(self.get_rows(&stmt.tid[i]));

            column_index_map = HashMap::<String, usize>::new();
            for column in right.columns.clone() {
                column_tablename_map.insert(column.name.clone(), stmt.tid[i].clone());
                column_index_map.insert(column.name.into(), columnindex);
                columnindex += 1;
            }
            name_column_map.insert(stmt.tid[i].clone(), column_index_map);
            stmt.alias.insert(stmt.tid[i].clone(), stmt.tid[i].clone());
            let tmp = try!(self.cross_rows(left, right));
            left = tmp;
        }
        masterrow = left;

        // compute conditions
        let mut whereresult = if stmt.cond.is_some() {
            println!("Wherestmt {:?}", stmt.cond );
                try!(self.execute_where(masterrow,
                    (&stmt.alias, &column_tablename_map, &name_column_map),
                    &stmt.cond.unwrap(), false, Where::Select))
            } else {
                masterrow
            };

        // the string will be but in front of the original rows name.
        // if bool = false. if bool = true the original columnname will be
        // overwritten
        let mut indextargets: Vec<((String, bool) , usize)> = Vec::new();
        for target in stmt.target {
            let rename = if target.rename.is_some() {
                let tmp = target.clone();
                tmp.rename.unwrap().clone()
            } else {
                "".into()
            };

            match target.col {
                Col::Every => {
                    if target.alias.is_some() {
                        let mut targetclone = target.clone();
                        let mut tablename = stmt.alias.get(&targetclone.alias.unwrap());
                        if tablename.is_none() {
                            return Err(ExecutionError::UnknownAlias)
                        }
                        let columntoindex = name_column_map.get(tablename.unwrap()).unwrap();
                        for index in columntoindex.values() {
                            targetclone = target.clone();
                            let append = if target.rename.is_some() {
                                (rename.clone(), true)
                            } else {
                                (format!("{}.", targetclone.alias.unwrap()), false)
                            };
                            indextargets.push((append, index.clone()));
                        };

                    } else {
                        for i in 0..(whereresult.columns.len()) {
                            let append = if target.rename.is_some() {
                                (rename.clone(),true)
                            } else {
                                ("".into(),false)
                            };
                            indextargets.push((append,i));
                        }

                    }
                },
                Col::Specified(column) => {
                    let tablename = if target.alias.is_some() {
                        stmt.alias.get(&target.alias.unwrap())
                    } else {
                        column_tablename_map.get(&column)
                    };
                    if tablename.is_none() {
                        return Err(ExecutionError::UnknownColumn)
                    }
                    let columntoindex = name_column_map.get(tablename.unwrap()).unwrap();
                    let column = columntoindex.get(&column);
                    if column.is_none() {
                        return Err(ExecutionError::UnknownColumn)
                    }
                    let append = if target.rename.is_some() {
                        (rename.clone(),true)
                    } else {
                        (format!("{}.",tablename.unwrap().clone()),false)
                    };
                    indextargets.push((append,column.unwrap().clone()));


                }
            }
        }

        try!(whereresult.reset_pos());
        let mut columnvec: Vec<Column> = Vec::new();
        for index in indextargets.clone() {
            whereresult.columns[index.1].name =
                if (index.0).1 {
                    (index.0).0
                } else {
                    format!("{}{}",(index.0).0, whereresult.columns[index.1].name)
                };

            columnvec.push(whereresult.columns[index.1].clone());
        }

        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut resultrows = Rows::<Cursor<Vec<u8>>>::new(cursor, &columnvec);

        // TODO: implement skiprow for Rows!!!
        // TODO: use less function calls of unwrap!!
        let mut limitcount = (false,0);
        if stmt.limit.is_some() {
            let limit = stmt.limit.unwrap();
            limitcount = (true,limit.count.unwrap().clone());
            if limit.offset.is_some() {
                for i in 0..limit.offset.unwrap() {
                    let mut skiprow =  Vec::<u8>::new();
                    match whereresult.next_row(&mut skiprow) {
                        Ok(_) => (),
                        Err(_) => break,
                    };
                }
            }
        }

        // TODO: Errormanagement!!!
        loop {
            if limitcount.0 && limitcount.1 == 0 {
                break
            }
            let mut originalrow =  Vec::<u8>::new();
            let res = whereresult.next_row(&mut originalrow);
            match res {
                Ok(_) => (),
                Err(_) => break,
            }
            let mut toinsert = Vec::<u8>::new();
            for index in indextargets.clone() {
                toinsert.extend(try!(whereresult.get_value(&originalrow,index.1)).into_iter());
            }
            resultrows.add_row(&toinsert);
            limitcount.1 -=1;

        }



        Ok(resultrows)
    }

    fn execute_where<'b>(&self,
          mut tableset:Rows<Cursor<Vec<u8>>>,
          infos: (&HashMap<String, String>,
                  &HashMap<String, String>,
                  &HashMap<String, HashMap<String, usize>>),
          conditions: &Conditions, negate: bool,
          wheretype: Where
        )
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {

        match conditions {


            &Conditions::And(ref c1, ref c2) => {
                if wheretype == Where::Select {
                    let mut leftside = try!(
                        self.execute_where(tableset, infos, c1, false, wheretype.clone()));
                    self.execute_where(leftside, infos, c2, false, wheretype)
                } else {


                    // IMPLEMENT!!! Needs a custom merge function
                    let mut rightresult = try!(
                        self.execute_where(try!(tableset.full_scan()),
                        infos, c1, true, Where::Select));
                    try!(self.execute_where(
                        tableset,
                        infos, c2, false, wheretype.clone()));
                    let mut engine = try!(self.get_engine(&wheretype.unwrap()));
                    try!(rightresult.reset_pos());
                    loop {
                        let mut rightrow = Vec::<u8>::new();
                        let outerres = rightresult.next_row(&mut rightrow);
                         match outerres {
                            Ok(_) => (),
                            Err(_) => break
                        }
                        engine.insert_row(&rightrow);
                    }


                    Ok(rightresult)

                }
            },





            &Conditions::Or(ref c1, ref c2) => {
                // When changing to the EFFECTIVE PLAN:
                // REMEMBER CHANGING HERE TOO! (TODO)
                if wheretype == Where::Select {
                    let tableset2 = try!(tableset.full_scan());
                    let leftside = try!(self.execute_where(tableset, infos, c1, false, wheretype.clone()));
                    let rightside = try!(self.execute_where(tableset2, infos, c2, false, wheretype));
                    self.merge_rows(leftside, rightside)
                }
                else {
                    try!(self.execute_where(try!(tableset.full_scan()), infos, c1, false, wheretype.clone()));
                    self.execute_where(tableset, infos, c2, false, wheretype)
                }

            },



            // TODO: SO MUCH REDUNDANT CODE!!!!!!!11111
            //       remove whenever there is time.
            &Conditions::Leaf(ref c) => {
                let tablename = if c.aliascol.is_some() {
                    match infos.0.get(&c.clone().aliascol.unwrap()) {
                        Some(x) => x,
                        None => return Err(ExecutionError::UnknownAlias)
                    }
                } else {
                    match infos.1.get(&c.col) {
                        Some(x) => x,
                        None => return Err(ExecutionError::UnknownColumn)
                    }
                };
                let columntoindex = infos.2.get(tablename).unwrap();
                let column = columntoindex.get(&c.col);
                if column.is_none() {
                    return Err(ExecutionError::UnknownColumn)
                }
                let index = column.unwrap().clone();

                match c.rhs {

                    CondType::Word(ref column) => {
                        let tablename2 = if c.aliasrhs.is_some() {
                            match infos.0.get(&c.clone().aliasrhs.unwrap()) {
                                Some(x) => x,
                                None => return Err(ExecutionError::UnknownAlias)
                            }
                        } else {
                            match infos.1.get(column) {
                                Some(x) => x,
                                None => return Err(ExecutionError::UnknownColumn)
                            }
                        };
                        let columntoindex2 = infos.2.get(tablename2).unwrap();
                        let column2 = columntoindex2.get(column);
                        if column2.is_none() {
                            return Err(ExecutionError::UnknownColumn)
                        }
                        let index2 = column2.unwrap().clone();
                        let operator = if negate {
                            c.op.negate()
                        } else {
                            c.op
                        };
                        if wheretype == Where::Select {
                            Ok(try!(tableset.lookup(index,
                                (&Vec::<u8>::new(), Some(index2)) , operator)))
                        } else {
                            let engine = try!(self.get_engine(&wheretype.unwrap()));
                            try!(engine.delete(index,
                                (&Vec::<u8>::new(), Some(index2)), operator));
                            Ok(generate_rows_dummy())
                        }

                    },

                    CondType::Literal(ref lit) => {
                        // Error handling: if wrong compare type is giving => Missmatch error
                        match tableset.columns[index].sql_type {
                            SqlType::Char(c) => if lit.sqltype() != SqlType::Char(0) {
                                return Err(ExecutionError::CompareDatatypeMissmatch)
                            },
                            _ => if tableset.columns[index].sql_type.clone() != lit.sqltype() {
                                 return Err(ExecutionError::CompareDatatypeMissmatch)
                            }
                        }
                        // TODO: use get_column methods!!
                        let mut comparedata = Vec::<u8>::new();
                        try!(tableset.columns[index].sql_type.encode_into(& mut comparedata,lit));
                        let operator = if negate {
                            c.op.negate()
                        } else {
                            c.op
                        };
                        if wheretype == Where::Select {
                            Ok(try!(tableset.lookup(index, (&comparedata, None) , operator)))
                        } else {
                            let engine = try!(self.get_engine(&wheretype.unwrap()));
                            engine.delete(index, (&comparedata, None), operator);
                            Ok(generate_rows_dummy())
                        }
                    },

                }


            },
        }

    }

    fn execute_delete_stmt(&mut self, mut query: DeleteStmt) -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError> {

        let mut table = try!(self.get_rows(&query.tid));
        let mut name_column_map = HashMap::<String, HashMap<String, usize>>::new();
        let mut column_index_map = HashMap::<String, usize>::new();
        let mut column_tablename_map = HashMap::<String, String>::new();
        let mut columnindex: usize = 0;
        for column in table.columns.clone() {
            column_tablename_map.insert(column.name.clone(), query.tid.clone());
            column_index_map.insert(column.name.into(), columnindex);
            columnindex += 1;
        }

        name_column_map.insert(query.tid.clone(), column_index_map);
        query.alias.insert(query.tid.clone(), query.tid.clone());

        if query.cond.is_some() {
            try!(self.execute_where(table,
                    (&query.alias, &column_tablename_map, &name_column_map),
                    &query.cond.unwrap(), false, Where::Delete(query.tid)));
        } else {
            let mut engine = try!(self.get_engine(&query.tid));
            // Todo: Storage: enable full table reset!!
            try!(engine.reset());
        }

        Ok(generate_rows_dummy())
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
            AlterOp::Add(columninfo) => {
                let mut table = try!(self.get_table(&stmt.tid));
                // Todo: no fullscan necessary!
                let mut rows = try!(self.get_rows(&stmt.tid));
                if !try!(rows.is_empty()) {
                    return Err(ExecutionError::TableNotEmpty)
                }

                let comment = if columninfo.comment.is_some() {
                    columninfo.comment.unwrap()
                } else {
                    "".into()
                };

                table.add_column(&columninfo.cid,
                                 columninfo.datatype,
                                 !columninfo.not_null,
                                 &comment,
                                 columninfo.primary
                                 );
                try!(table.save());
                Ok(generate_rows_dummy())
            },
            AlterOp::Drop(column) => {
                let mut table = try!(self.get_table(&stmt.tid));
                // Todo: no fullscan necessary!
                let mut rows = try!(self.get_rows(&stmt.tid));
                if !try!(rows.is_empty()) {
                    return Err(ExecutionError::TableNotEmpty)
                }
                table.remove_column(&column);
                try!(table.save());
                Ok(generate_rows_dummy())
            },
            AlterOp::Modify(columninfo) => {
                let mut table = try!(self.get_table(&stmt.tid));
                {
                let columns = &mut table.meta_data.columns;
                let comment = if columninfo.comment.is_some() {
                    columninfo.comment.unwrap()
                } else {
                    "".into()
                };

                for index in 0..columns.len() {
                    if columns[index].name == columninfo.cid {
                        columns[index] = Column {
                            name: columninfo.cid.clone(),
                            sql_type: columninfo.datatype,
                            is_primary_key: columninfo.primary,
                            allow_null: !columninfo.not_null,
                            description: comment.clone()
                        };
                    }
                }
                }
                //println!("{:?}",table);
                try!(table.save());
                Ok(generate_rows_dummy())
            },
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
        let mut rows = try!(engine.full_scan());
        try!(rows.reset_pos());
        Ok(rows)
    }

    fn merge_rows(&self, mut left: Rows<Cursor<Vec<u8>>>, mut right: Rows<Cursor<Vec<u8>>>)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        try!(right.reset_pos());
        loop {
            try!(left.reset_pos());
            let mut valid = true;

            let mut rightrow = Vec::<u8>::new();
            let outerres = right.next_row(&mut rightrow);
             match outerres {
                Ok(_) => (),
                Err(_) => break
            }
            loop {
                let mut leftrow = Vec::<u8>::new();
                let outerres = left.next_row(&mut leftrow);
                 match outerres {
                    Ok(_) => (),
                    Err(_) => break
                }
                            let mut primarykeys = 0;
            let mut equalprimarykeys = 0;
                for index in 0..left.columns.len() {
                    if left.columns[index].is_primary_key {
                        primarykeys += 1;
                        let leftval = try!(left.get_value(&leftrow,index));
                        let rightval = try!(right.get_value(&rightrow,index));
                        if leftval == rightval {
                            equalprimarykeys +=1;
                        }
                    }
                }
                if primarykeys==equalprimarykeys {
                    valid = false;
                }
            }
            if valid {
                left.add_row(&rightrow);
            }
        }

        Ok(left)


    }


    fn cross_rows(&self, mut left: Rows<Cursor<Vec<u8>>>, mut right: Rows<Cursor<Vec<u8>>>)
        -> Result<Rows<Cursor<Vec<u8>>>, ExecutionError>
    {
        try!(left.reset_pos());
        try!(right.reset_pos());
        let mut columnvec = left.columns.clone();
        let mut appendvec = right.columns.clone();

        for i in 0..appendvec.len() {
            let appendlength = appendvec.len();
            columnvec.push(appendvec.remove(0));
        }
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut rows = Rows::<Cursor<Vec<u8>>>::new(cursor, &columnvec);


        loop {
            let mut insertingrow = Vec::<u8>::new();
            let outerres = left.next_row(&mut insertingrow);

            match outerres {
                Ok(_) => (),
                Err(_) => break
            }
                loop {
                    let mut datasrc = Vec::<u8>::new();
                    for i in 0..insertingrow.len() {
                        datasrc.push(insertingrow[i]);
                    }
                    let innerres = right.next_row(&mut datasrc);
                    match innerres {
                        Ok(_) => {
                            try!(rows.add_row(& datasrc));
                            ()
                        },
                        Err(_) => {
                            right.reset_pos();
                            break
                        }
                    }
                }

        }

        Ok(rows)
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
    UnknownAlias,
    UnknownColumn,
    CompareDatatypeMissmatch,
    TableNotEmpty,
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

#[derive(PartialEq, Clone)]
pub enum Where {
    Select,
    Delete(String)
}

impl Where {
    pub fn unwrap(&self ) -> String {
        match self {
            &Where::Select => "".into(),
            &Where::Delete(ref s) => s.clone(),
        }
    }
}
