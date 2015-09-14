extern crate server;
use server::storage::bstar::{ Bstar, Bnode, SortedList, KeyAddr };
use std::fs::File;
use std::io::*;
use std::fs::OpenOptions;

fn main() {
    Bstar::<u64>::delete("test");
    let mut _tree = Bstar::create("test", 2);
    let mut tree = match _tree {
        Ok(t) => t,
        _ => panic!("error"),
    };
    println!("Insert PRINT:");
    println!("");

    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(1,2)));
    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(2,2)));
    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(3,2)));
    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(4,2)));

    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(6,2)));
    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(7,2)));
    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(8,2)));
    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(9,2)));
    println!("Inserted res = {:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(5,2)));
    let mut _tree = Bstar::<u64>::load("test");
    let mut tree = match _tree {
        Ok(t) => t,
        Err(e) => panic!(format!("{:?}",e)),
    };
    println!("");
    println!("DEBUG PRINT:");
    println!("");
    tree.debug_print();


    let mut list = SortedList::<u64>::new();
    list.insert(1);
    list.insert(4);
    list.insert(8);

    list.insert(5);

    println!("{:?}", list);


    }
