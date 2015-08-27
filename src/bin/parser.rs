/// Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;

fn main() {
    println!("{:?}", parse::parse("hi"));
}
