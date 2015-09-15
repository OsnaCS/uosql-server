extern crate server;
use std::io::{self, stdout, Write};
use server::parse;
use server::query;
use server::auth;


fn main() {
    print!("Username: ");
    let username = read_query();
    let mut user = auth::User { _name: username.into(), _currentDatabase: None };
    println!("to exit program type 'exit'");
    print!("Sql Query: ");
    let mut query = read_query();
    while query != "exit" {
        //execute(&query, & mut user);
        //print!("Sql Query: ");
        //query = read_query();
    }

}


fn execute(query: &str, user: & mut auth::User) {
        let ast = parse::parse(query);

        match ast {
        Ok(tree) => {
                println!("{:?}", tree);
                /*match query::execute_from_ast(tree, user) {
                    Ok(s) => println!("Resultset: {:?}", s),
                    Err(error) => println!("{:?}", error),
                };*/
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
