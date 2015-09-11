extern crate server;
use server::storage::bstar::{ Bstar, Bnode };
use std::fs::File;
use std::io::*;
use std::fs::OpenOptions;

fn main() {

    /*let tree = Bnode { size: 10 };

    let mut _file = OpenOptions::new().write(true).read(true).create(true).open("star.b");

    let mut file = match _file {
        Ok(f) => f,
        Err(err) => panic!("FileError"),
    };

    println!("{:?}",file.write(&[5,6,4,5,6]));

    file.seek(SeekFrom::Start(4));

    let mut result = [0];
    let res = file.read(&mut result);

    println!("{:?}, {:?}",res,  result );

    /*let mut file = try!(OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(&self.table.get_table_data_pat


                            h()));*/
    */


    let mut tree = Bstar::create("star.b", 10).unwrap();
    tree.insert(111,222);
    println!("{:?}", tree);
    tree = Bstar::load("star.b").unwrap();
    println!("{:?}", tree);
    println!("{:?}", tree.get_root() );

}
