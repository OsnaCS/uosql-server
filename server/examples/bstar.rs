extern crate server;
use server::storage::bstar::{ Bstar, Bnode, SortedList, KeyAddr };
use std::fs::File;
use std::io::*;
use std::fs::OpenOptions;

fn main() {
    Bstar::<u64>::delete("test");
    let mut _tree = Bstar::create("test", 70);
    let mut tree = match _tree {
        Ok(t) => t,
        _ => panic!("error"),
    }; 
    for i in 0..102 {
       let key =  KeyAddr::<u64>::new(105-i,9483259);
        tree.insert_keyaddr(key);
    }  
    let mut _tree = Bstar::<u64>::load("test");
    let mut tree = match _tree {
        Ok(t) => t,
        Err(e) => panic!(format!("{:?}",e)),
    };

    println!("{:?}", tree.get_root() );



    }
