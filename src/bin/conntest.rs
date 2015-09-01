extern crate uosql;
use uosql::net;
use uosql::net::ToNetwork;
fn main(){
	println!("test greeting struct");
	testgreeting();

}

fn testgreeting(){
	let greet = net::Greeting::make_greeting(1, "willkommen".to_string());
	println!("version: {}\nmessage size: {}\nmessage: {}", greet.protocol_version, greet.size_of_message, greet.message );

	println!("test of write: ");
	let mut vec = Vec::new();

	greet.write(&mut vec);
	println!("{:?} ", vec);
	
}