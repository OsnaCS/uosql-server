extern crate server;
use std::io::{self, stdout, Write};
use server::parse;
use server::query;
use server::auth;
use server::net;
use server::net::types::DataSet;
use server::storage::{ResultSet, SqlType};
use std::cmp::{max, min};


fn main() {

    print!("Username: ");
    let username = read_query();
    let mut user = auth::User { _name: username.into(), _currentDatabase: None };
    println!("to exit program type 'exit'");
    print!("Sql Query: ");
    let mut query = read_query();
    while query != "exit" {
        execute(&query, & mut user);
        print!("Sql Query: ");
        query = read_query();
    }

}


fn execute(query: &str, user: & mut auth::User) {
        let ast = parse::parse(query);

        match ast {
        Ok(tree) => {
                println!("{:?}", tree);
                match query::execute_from_ast(tree, user) {
                    Ok(s) => display(&mut net::types::preprocess(&s)),
                    Err(error) => println!("{:?}", error),
                };
            },
        Err(error) => println!("{:?}", error),
}
}

pub fn read_query() -> String {
        let e = stdout().flush();
        match e {
            Ok(_) => {},
            Err(_) => {},
        }
        let a = read_line();
        return a;
}


fn read_line() -> String {
    let mut input = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                match &*input {
                    "\n" => return input,
                    _ => return input.trim().into()
                }
            },
            _ => { }
        }
    }
}

pub fn display(table: &mut DataSet) {
    if table.data_empty() && table.metadata_empty() {
        // println!("done.");
    } else if table.data_empty() {
        display_meta(table)
    } else {
        display_data(table)
    }
}

/// Formated display of table data.
fn display_data(table: &mut DataSet) {
    let mut cols = vec![];

    for i in 0..table.get_col_cnt() {
        match table.get_type_by_idx(i).unwrap_or(SqlType::Int) {
            SqlType::Int => {
                cols.push(max(12, table.get_col_name(i).unwrap().len()));
            },
            SqlType::Bool => {
                cols.push(max(5, table.get_col_name(i).unwrap().len()));
            },
            SqlType::Char(size) => {
                cols.push(max(size as usize, table.get_col_name(i).unwrap().len()));
            }
        }
    }

    // column names
    display_seperator(&cols);

    for i in 0..(cols.len()) {
        if table.get_col_name(i).unwrap().len() > 27 {
            print!("| {}... ", &table.get_col_name(i).unwrap_or("none")[..27]);
        } else {
            print!("| {1: ^0$} ", min(30, cols[i]), table.get_col_name(i).unwrap_or("none"));
        }
    }
    println!("|");

    display_seperator(&cols);

    // display actual data
    while table.next()
    {
        for i in 0..(cols.len()) {
            // println!("i = {:?}", i);
            match table.get_type_by_idx(i) {
                Some(t) => {
                    match t {
                        SqlType::Int =>
                            match table.next_int_by_idx(i) {
                                Some(val) =>
                                    print!("| {1: ^0$} ", min(30, cols[i]), val),
                                None => print!("| {1: ^0$} ", min(30, cols[i]), "none"),
                            },
                        SqlType::Bool =>
                            match table.next_bool_by_idx(i) {
                                Some(val) =>
                                    print!("| {1: ^0$} ", min(30, cols[i]), val),
                                None => print!("| {1: ^0$} ", min(30, cols[i]), "none"),
                            },
                        SqlType::Char(_) =>
                            print!("| {1: ^0$} ", min(30, cols[i]),
                                    table.next_char_by_idx(i).unwrap_or("none".into()))
                    }
                },
                None => continue
            }
        }
        println!("|");
    }
    display_seperator(&cols);
}

/// Formated display of MetaData.
fn display_meta(table: &mut DataSet) {
    // print meta data
    let mut cols = vec![];
    for i in 0..table.get_col_cnt() {
        cols.push(max(12, max(table.get_col_name(i).unwrap().len(),
                        table.get_description_by_idx(i).unwrap_or("none").len())));
    }

    // Column name +---
    print!("+");
    let col_name = "Column name";
    for _ in 0..(col_name.len()+2) {
        print!("-");
    }

    // for every column +---
    display_seperator(&cols);

    print!("| {} ", col_name);
    // name of every column
    for i in 0..(cols.len()) {
        if table.get_col_name(i).unwrap_or("none").len() > 27 {
            print!("| {}... ", &table.get_col_name(i).unwrap_or("none")[..27]);
        } else {
            print!("| {1: ^0$} ", min(30, cols[i]), table.get_col_name(i).unwrap_or("none"));
        }
    }
    println!("|");

    // format +--
    print!("+");
    for _ in 0..(col_name.len()+2) {
        print!("-");
    }

    display_seperator(&cols);

    print!("| {1: <0$} ", col_name.len(), "Type");
    for i in 0..(cols.len()) {
        match table.get_type_by_idx(i) {
            Some(t) => print!("| {1: ^0$} ", min(30, cols[i]), format!("{:?}", t)),
            None => print!("| {1: ^0$} ", min(30, cols[i]), "none")
        }
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Primary");
    for i in 0..(cols.len()) {
        match table.get_is_primary_key_by_idx(i) {
            Some(t) => print!("| {1: ^0$} ", min(30, cols[i]), format!("{:?}", t)),
            None => print!("| {1: ^0$} ", min(30, cols[i]), "none")
        }
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Allow NULL");
    for i in 0..(cols.len()) {
        match table.get_allow_null_by_idx(i) {
            Some(t) => print!("| {1: ^0$} ", min(30, cols[i]), t),
            None => print!("| {1: ^0$} ", min(30, cols[i]), "none")
        }
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Description");
    for i in 0..(cols.len()) {
        if table.get_description_by_idx(i).unwrap().len() > 27 {
            //splitten
            print!("| {}... ", &table.get_description_by_idx(i).unwrap()[..27]);
        } else {
            print!("| {1: ^0$} ", min(30, cols[i]),
                            table.get_description_by_idx(i).unwrap_or("none"));
        }
    }
    println!("|");

    print!("+");
    for _ in 0..(col_name.len()+2) {
        print!("-");
    }

    display_seperator(&cols);
}

/// Display separator line adjusted to given column sizes. (Pattern +-...-+)
pub fn display_seperator(cols: &Vec<usize>) {
    for i in 0..(cols.len()) {
        print!("+--");
        for _ in 0..min(30, cols[i]) {
            print!("-");
        }
    }
    println!("+");
}
