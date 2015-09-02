/// Program for testing and playing with the parser
///

extern crate uosql;
use uosql::parse;

fn main() {
	let l = parse::lex::Lexer::from_query("Create table personen (name_v var(256), name_n var(256), alter char(16));");
	let m = parse::lex::Lexer::from_query("Select * from personen where \"alter\" >= 0.1;");
	for i in m {
		println!("{:?} ", i.tok);
	}
}
