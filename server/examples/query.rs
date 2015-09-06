extern crate server;
use server::parse;
use server::query;


fn main() {


    let mut p = parse::Parser::create("create table testtable");
    let ast = p.parse();

    match ast {
        Ok(tree) => {
                println!("{:?}", tree);
                query::execute_from_ast(tree, Some("testbase".into()));
            },
        Err(error) => println!("{:?}", error),
}

}
