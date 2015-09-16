extern crate server;
use std::io::{self, stdout, Write};
use server::parse;
use server::query;
use server::auth;
use server::storage::{ResultSet};
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
                    Ok(s) => display(&s),
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


/// Display data from ResultSet.
pub fn display(row: &ResultSet) {
    if row.data.is_empty() && row.columns.is_empty() {
        println!("No data to display received.");
    } else if row.data.is_empty() {
        display_meta(&row)
    } else {
        display_data(&row)
    }
}

/// Formated display of table data.
fn display_data(row: &ResultSet) {
    let mut cols = vec![];
    for i in &row.columns {
        cols.push(max(12, i.name.len()));
    }

    // column names
    display_seperator(&cols);

    for i in 0..(cols.len()) {
        if row.columns[i].name.len() > 27 {
            print!("| {}... ", &row.columns[i].name[..27]);
        } else {
            print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].name);
        }
    }
    println!("|");

    display_seperator(&cols);
}

/// Formated display of MetaData.
fn display_meta(row: &ResultSet) {
    // print meta data
    let mut cols = vec![];
    for i in &row.columns {
        cols.push(max(12, max(i.name.len(), i.description.len())));
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
        if row.columns[i].name.len() > 27 {
            print!("| {}... ", &row.columns[i].name[..27]);
        } else {
            print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].name);
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
        print!("| {1: ^0$} ", min(30, cols[i]), format!("{:?}", row.columns[i].sql_type));
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Primary");
    for i in 0..(cols.len()) {
        print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].is_primary_key);
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Allow NULL");
    for i in 0..(cols.len()) {
        print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].allow_null);
    }
    println!("|");

    print!("| {1: <0$} ", col_name.len(), "Description");
    for i in 0..(cols.len()) {
        if row.columns[i].description.len() > 27 {
            //splitten
            print!("| {}... ", &row.columns[i].description[..27]);
        } else {
            print!("FALSE");
            print!("| {1: ^0$} ", min(30, cols[i]), row.columns[i].description);
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
