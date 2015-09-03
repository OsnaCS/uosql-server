/// Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;

fn main() {
	let l = parse::lex::Lexer::from_query(" ");
	let m = parse::lex::Lexer::from_query("\'10.08.1991\" and this does not belong to it");
	for i in l {
		println!("{:?} ", i.tok);
	}
}
