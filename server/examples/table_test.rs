extern crate server;
extern crate bincode;
extern crate log;

use server::storage::*;
use bincode::rustc_serialize::{encode_into};
use bincode::SizeLimit;
use server::logger;
use server::parse::ast::DataSrc;

fn main() {

    logger::with_loglevel(::log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();

    let ty = SqlType::Int;
    let mut v = Vec::new();
    let _ = encode_into(&ty, &mut v, SizeLimit::Infinite);

    //let db = Database::create("storage_team").unwrap();
    let db = Database::load("storage_team").unwrap();

    let mut cols = Vec::new();
    cols.push(Column {
        name: "Heiner".into(),
        sql_type: SqlType::Int,
        allow_null: false,
        description: "Heiner".to_string(),
        is_primary_key: false,
    });
    cols.push(Column {
        name: "Mathias".into(),
        sql_type: SqlType::Bool,
        allow_null: false,
        description: "Mathias".to_string(),
        is_primary_key: false,
    });
    cols.push(Column {
        name: "Dennis".into(),
        sql_type: SqlType::Char(6),
        allow_null: false,
        description: "Dennis".to_string(),
        is_primary_key: true,
    });
    cols.push(Column {
        name: "Jana".into(),
        sql_type: SqlType::VarChar(178),
        allow_null: false,
        description: "Jana".to_string(),
        is_primary_key: false,
    });

    let mut my_data: Vec<Option<DataSrc>> = Vec::new();
    my_data.push(Some(DataSrc::Int(10)));
    my_data.push(Some(DataSrc::Bool(1)));
    my_data.push(Some(DataSrc::String("fuenf".to_string())));
    my_data.push(
        Some(DataSrc::String("i am a very long string, at least i think i am".to_string()))
    );

    let _storage_team = db.create_table("storage_team", cols, 1).unwrap();

    let t = db.load_table("storage_team").unwrap();

    let descriptions = t.get_description().unwrap();

    for i in descriptions.iter() {
        print!("{}", i.get_value::<String>(0).unwrap());
        print!(", {}", i.get_value::<String>(1).unwrap());
        print!(", {}", i.get_value::<bool>(2).unwrap());
        print!(", {}", i.get_value::<bool>(3).unwrap());
        println!(", {}", i.get_value::<String>(4).unwrap());
    }

    let mut engine = t.create_engine();
    //engine.create_table();
    engine.insert_row(&my_data);
    let rows = engine.full_scan().unwrap();


    for i in rows.iter() {
        print!("{}", i.get_value::<i32>(0).unwrap());
        print!(", {}", i.get_value::<bool>(1).unwrap());
        print!(", {}", i.get_value::<String>(2).unwrap());
        println!(", {}", i.get_value::<String>(3).unwrap());
    }









    //let _e  = engine.create_table();
    //let t = engine.table();

    //t.delete().unwrap();
    //db.delete().unwrap();

}
