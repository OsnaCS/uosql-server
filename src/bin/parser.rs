/// Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;

fn main() {
	let l = parse::lex::Lexer::from_query("hi, Lisa! Wie geht es dir?");
	for i in l {
		println!("{:?}", i.tok);
	}
}
