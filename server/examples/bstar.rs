extern crate server;
use server::storage::bstar::{ Bstar, Bnode, SortedList };
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

    /*
    let mut tree = Bstar::create("star.b", 10).unwrap();
    tree.insert(111,222);
    println!("{:?}", tree);
    tree = Bstar::load("star.b").unwrap();
    println!("{:?}", tree);
    println!("{:?}", tree.get_root() );
    */

    let mut list = SortedList::new();

    list.insert(5);
    println!("{:?}",list.delete(&5));
    println!("{:?}", list);
    println!("{:?}", list.insert(5));
    println!("{:?}", list.insert(5));
     println!("{:?}", list.insert(2));
     for i in 0..100 {
        list.insert(100-i);

     }

    println!("{:?}",list.delete(&37) );
    println!("{:?}", list);
    println!("{:?}",list.delete(&37) );
    println!("{:?}", list);
}
