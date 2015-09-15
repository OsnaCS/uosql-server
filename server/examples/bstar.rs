extern crate server;
use server::storage::bstar::{ Bstar, Bnode, SortedList, KeyAddr };
use std::fs::File;
use std::io::*;
use std::fs::OpenOptions;

fn main() {
    Bstar::<u64>::delete("test");

    let mut _tree = Bstar::create("test", "TARGETTABLEyxcyxvxcxyxc",2);
    let mut tree = match _tree {
        Ok(t) => t,
        _ => panic!("error"),
    };

    tree.insert_keyaddr(KeyAddr::<u64>::new(1,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(2,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(3,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(4,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(6,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(7,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(10,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(9,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(11,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(8,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(5,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(12,2));
    tree.insert_keyaddr(KeyAddr::<u64>::new(13,2));



    /*for i in 32..85 {
        tree.insert_keyaddr(KeyAddr::<u64>::new(i,1000-i));
    }

    for i in 194..200 {
        tree.insert_keyaddr(KeyAddr::<u64>::new(i,5*i));
    }

    for i in 1..18 {
        tree.insert_keyaddr(KeyAddr::<u64>::new(194-i,10*i));
    }
        tree.insert_keyaddr(KeyAddr::<u64>::new(0,2));
    */
    let mut _tree = Bstar::<u64>::load("test");
    tree = match _tree {
        Ok(t) => t,
        Err(e) => panic!(format!("{:?}",e)),
    };
    println!("");
    println!("DEBUG PRINTING {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");

    println!("{:?}", tree.delete_keyaddr(1));
        println!("");
    println!(" {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");
    println!("{:?}", tree.delete_keyaddr(2));
        println!("");
    println!(" {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");
    println!("{:?}", tree.delete_keyaddr(3));
        println!("");
    println!(" {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");
    println!("{:?}", tree.delete_keyaddr(7));

    println!(" {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");
    println!("{:?}", tree.delete_keyaddr(12));

    println!(" {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");
        println!("{:?}", tree.delete_keyaddr(9));

    println!(" {:?} ELEMENTS:", tree);
    println!("");
    tree.debug_print();
    println!("");
    println!("");


    for keyaddr in tree.iter_start_at(8) {
        println!("{:?}", keyaddr );
    }

    }
