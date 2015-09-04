extern crate uosql;
extern crate bincode;

use uosql::storage::*;
use bincode::rustc_serialize::{encode_into};
use bincode::SizeLimit;
use uosql::parse::ast::SqlType;

fn main() {

    let ty = SqlType::Int;
    let mut v = Vec::new();
    encode_into(&ty, &mut v, SizeLimit::Infinite);
    println!("{:?}", v);

    //let db = Database::create("storage_team").unwrap();
    let db = Database::load("storage_team").unwrap();

    let mut cols = Vec::new();
    cols.push(Column { name: "Heiner".into(), sql_type: SqlType::Int });
    cols.push(Column { name: "Mathias".into(), sql_type: SqlType::Bool });
    cols.push(Column { name: "Dennis".into(), sql_type: SqlType::Char(6) });
    cols.push(Column { name: "Jana".into(), sql_type: SqlType::VarChar(178) });


    let _storage_team = db.create_table("storage_team", cols, 1).unwrap();

    let t = db.load_table("storage_team").unwrap();

    let mut engine = t.create_engine();
    let _e  = engine.create_table();
    // println!("{:?}", t.name);
    let f = engine.table();
    println!("{:?}", f.name);
    println!("{:?}", f);
}
