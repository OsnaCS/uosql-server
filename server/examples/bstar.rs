extern crate server;
use server::storage::bstar::{ Bstar, Bnode, SortedList, KeyAddr };
use std::fs::File;
use std::io::*;
use std::fs::OpenOptions;

fn main() {
    Bstar::<u64>::delete("test");
    let mut _tree = Bstar::create("test", 3);
    let mut tree = match _tree {
        Ok(t) => t,
        _ => panic!("error"),
    };

    println!("{:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(1,2)));
    tree.insert_keyaddr(KeyAddr::<u64>::new(2,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(3,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(4,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(6,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(7,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(10,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(9,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(11,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(0,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(8,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(5,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(12,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(13,2));

    for i in 32..85 {
        tree.insert_keyaddr(KeyAddr::<u64>::new(i,2));
    }

    for i in 194..225 {
        println!("{:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(i,2)));
    }

    for i in 1..18 {
        println!("{:?}",tree.insert_keyaddr(KeyAddr::<u64>::new(194-i,2)));
    }
    println!("");
    println!("DEBUG PRINTINT {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    drop(tree);
    let mut _tree = Bstar::<u64>::load("test");
    tree = match _tree {
        Ok(t) => t,
        Err(e) => panic!(format!("{:?}",e)),
    };
    println!("");
    println!("DEBUG PRINTINT {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");



    }
