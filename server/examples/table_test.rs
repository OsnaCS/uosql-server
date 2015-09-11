extern crate server;
extern crate bincode;
extern crate log;

use server::storage::*;
use bincode::rustc_serialize::{encode_into};
use bincode::SizeLimit;
use server::logger;
use server::parse::ast::DataSrc;
use std::io::Cursor;

fn main() {

    logger::with_loglevel(::log::LogLevelFilter::Trace)
        .with_logfile(std::path::Path::new("log.txt"))
        .enable().unwrap();

    let ty = SqlType::Int;
    let mut v = Vec::new();
    let _ = encode_into(&ty, &mut v, SizeLimit::Infinite);

    //let db = Database::create("storage_team").unwrap();
    //let db = Database::load("storage_team").unwrap();

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

    let mut my_data: Vec<Option<DataSrc>> = Vec::new();
    my_data.push(Some(DataSrc::Int(10)));
    my_data.push(Some(DataSrc::Bool(1)));
    my_data.push(Some(DataSrc::String("sechs".to_string())));
    my_data.push(
        Some(DataSrc::String("i am a very long string, at least i think i am".to_string()))
    );

   // let _storage_team = db.create_table("storage_team", cols, 1).unwrap();

   // let t = db.load_table("storage_team").unwrap();


    //RRROOOOWWWWSSS
    let v = Vec::<u8>::new();
    let c = Cursor::new(v);

    let rows = Rows::new(c, &cols);
    //rows.add_row(vec![]).unwrap();


    //_type_test();
}


fn _type_test(){
    let my_int: [u8; 4] = [20, 30, 40, 50]; //0001:0100:0001:1110:0010:1000:0011:0010 -> 337520690
    let my_other_int: [u8; 4] = [10, 5, 0, 2];//0000:1010:0000:0101:0000:0000:0000:0010 -> 168099842

    let my_bool: [u8; 3] = [0, 0, 0];
    let my_other_bool: [u8; 3] = [0, 0, 0];

    let my_strin: [u8; 2] = [0, 41];
    let my_other_strin: [u8; 2] = [0, 41];

    let my_sqltype_int = SqlType::Int;
    let my_sqltype_bool = SqlType::Bool;
    let my_sqltype_char = SqlType::Char(7);
    println!("check for int:");

    println!("is equal: {:?}",my_sqltype_int.cmp(
        &my_int[0..4], &my_other_int[0..4], CompType::Equ).unwrap()
    );
    println!("is not equal: {:?}",my_sqltype_int.cmp(
        &my_int[0..4], &my_other_int[0..4], CompType::NEqu).unwrap()
    );
    println!("is greater: {:?}",my_sqltype_int.cmp(
        &my_int[0..4], &my_other_int[0..4], CompType::GThan).unwrap()
    );
    println!("is lesser: {:?}",my_sqltype_int.cmp(
        &my_int[0..4], &my_other_int[0..4], CompType::SThan).unwrap()
    );
    println!("is greater equal: {:?}",my_sqltype_int.cmp(
        &my_int[0..4], &my_other_int[0..4], CompType::GEThan).unwrap()
    );
    println!("is lesser equal: {:?}",my_sqltype_int.cmp(
        &my_int[0..4], &my_other_int[0..4], CompType::SEThan).unwrap()
    );

    println!("check for bool:");
    println!("is equal: {:?}",my_sqltype_bool.cmp(
        &my_bool[0..3], &my_other_bool[0..3], CompType::Equ).unwrap()
    );
    println!("is not equal: {:?}",my_sqltype_bool.cmp(
        &my_bool[0..3], &my_other_bool[0..3], CompType::NEqu).unwrap()
    );

    println!("check for string:");
    println!("is equal: {:?}",my_sqltype_char.cmp(
        &my_strin[0..2], &my_other_strin[0..2], CompType::Equ).unwrap()
    );
    println!("is not equal: {:?}",my_sqltype_char.cmp(
        &my_strin[0..2], &my_other_strin[0..2], CompType::NEqu).unwrap()
    );

}
