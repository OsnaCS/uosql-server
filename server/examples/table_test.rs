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

    let db = Database::create("storage_team").unwrap();
    //let db = Database::load("storage_team").unwrap();

    let mut cols = Vec::new();
    cols.push(Column {
        name: "Heiner".into(),
        sql_type: SqlType::Int,
        allow_null: false,
        description: "Heiner".to_string(),
        is_primary_key: true,
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
        is_primary_key: false,
    });

    let mut my_data: Vec<Option<DataSrc>> = Vec::new();
    my_data.push(Some(DataSrc::Int(10)));
    my_data.push(Some(DataSrc::Bool(1)));
    my_data.push(Some(DataSrc::String("sechs".to_string())));

    let _storage_team = db.create_table("storage_team", cols, EngineID::FlatFile).unwrap();

    let _storage_team = db.load_table("storage_team").unwrap();


    //RRROOOOWWWWSSS
    let v = Vec::<u8>::new();
    let c = Cursor::new(v);
    let data = vec![1, 2, 3, 4, 1, 0x48, 0x41, 0x4C, 0x4C, 0x4F, 0x00];

    let mut rows = Rows::new(c, &_storage_team.columns());
    rows.add_row(&data).unwrap();
    rows.reset_pos().unwrap();

    let mut d = Vec::<u8>::new();

    rows.next_row(&mut d).unwrap();
    println!{"{:?}", d}

    let h = rows.get_value(&d, 2);
    println!{"{:?}", h}

    rows.delete_row().unwrap();

    rows.reset_pos().unwrap();

    d.clear();

    flat_file_test();

}

fn flat_file_test() {
    println!("start flat file test");

    let data = Database::load("storage_team").unwrap();

    {
        let t = data.load_table("storage_team").unwrap();
        let mut engine = FlatFile::new(t);
        engine.create_table().unwrap();
        let mut rnd_data = vec![0, 0, 0, 1, 0, 0x48, 0x41, 0x4C, 0x4C, 0x4F, 0x00];

        engine.insert_row(&rnd_data).unwrap();
        rnd_data = vec![0, 0, 0, 2, 0, 0x48, 0x41, 0x4C, 0x4C, 0x4F, 0x00];
        engine.insert_row(&rnd_data).unwrap();

        rnd_data = vec![0, 0, 0, 3, 0, 0x48, 0x41, 0x4C, 0x4D, 0x4F, 0x00];
        engine.insert_row(&rnd_data).unwrap();

        rnd_data = vec![0, 0, 0, 4, 1, 0x48, 0x41, 0x4C, 0x4D, 0x4F, 0x00];
        engine.insert_row(&rnd_data).unwrap();
    }

    let t = data.load_table("storage_team").unwrap();
    let mut engine = FlatFile::new(t);

    // delete with bool
    let my_bool: [u8; 1] = [0x01];
    engine.delete(1, &my_bool[..], CompType::Equ).unwrap();

    // delete with char
    // let my_char: [u8; 6] = [0x48, 0x41, 0x4C, 0x4D, 0x4F, 0x00];
    // println!("{:?}", my_char);
    // engine.delete(2,&my_char[0..6],CompType::Equ).unwrap();

    println!("///////////////////////////////////////////////////////////////");
    println!("/////////////////////// Lookup ////////////////////////////////");
    println!("///////////////////////////////////////////////////////////////");

    let my_int: [u8; 4] = [0, 0, 0, 1];
    let mut rows = engine.lookup(0,&my_int[0..4],CompType::Equ).unwrap();
    rows.reset_pos().unwrap();

    println!("engine.lookup rows: {:?}", rows);

    println!("///////////////////////////////////////////////////////////////");
    println!("//////////////////////// Delete ///////////////////////////////");
    println!("///////////////////////////////////////////////////////////////");

    engine.delete(0,&my_int[0..4],CompType::Equ).unwrap();
    rows.reset_pos().unwrap();

    rows = engine.full_scan().unwrap();
    println!("the rows: {:?}", rows);
    rows.reset_pos().unwrap();

    // modify a char
    let my_char: [u8; 6] = [0x48, 0x41, 0x4C, 0x4D, 0x4F, 0x00];
    let my_char2: [u8; 6] = [0x48, 0x41, 0x4C, 0x4c, 0x4C, 0x00];
    let values = [(2 as usize, &(my_char2[..]))];
    println!("///////////////////////////////////////////////////////////////");
    println!("//////////////////////// Modify ///////////////////////////////");
    println!("///////////////////////////////////////////////////////////////");
    engine.modify(2, &my_char, CompType::Equ, &values).unwrap();

    rows.reset_pos().unwrap();

    rows = engine.full_scan().unwrap();
    println!("the rows: {:?}", rows);
    //rows.reset_pos().unwrap();

    rows = engine.full_scan().unwrap();
    println!("the rows2: {:?}", rows);

    let mut cols = Vec::new();
    cols.push(Column {
        name: "Heiner".into(),
        sql_type: SqlType::Char(6),
        allow_null: false,
        description: "Heiner".to_string(),
        is_primary_key: true,
    });

    let db = Database::create("test").unwrap();
    let _test = db.create_table("test", cols, EngineID::FlatFile).unwrap();

    let mut engine = FlatFile::new(_test);
    engine.create_table().unwrap();

    let mut rnd_data = vec![0x48, 0x41, 0x4C, 0x4C, 0x4F, 0x00];
    engine.insert_row(&rnd_data).unwrap();


    rows = engine.full_scan().unwrap();
    println!("the rows: {:?}", rows);

    rnd_data = vec![0x48, 0x41, 0x4C, 0x4C, 0x4D, 0x00];
    engine.insert_row(&rnd_data).unwrap();

    let rows2 = engine.full_scan().unwrap();
    println!("the rows2: {:?}", rows2);
}

fn _type_test() {
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
