/// Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;

fn main() {
	let l = parse::lex::Lexer::from_query("Create table personen (name_v var(256), name_n var(256), alter char(16));");
	let m = parse::lex::Lexer::from_query("\'10.08.1991\" and this does not belong to it");
	for i in m {
		println!("{:?} ", i.tok);
	}
}
