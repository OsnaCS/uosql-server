
use std::io::*;
use std::fs::File;
use std::fs;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::fs::OpenOptions;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::marker::PhantomData;

pub trait KnownSize {
    fn size() -> u64;
    fn read(&mut File, Option<u64>) -> Result<Self>;
    fn write(&self, &mut File, Option<u64>) -> Result<()>;
    fn write_default(&mut File, Option<u64>) -> Result<()>;
}

const FreeAdrr: u64 = 24;
const EoF: u64 = 32;
const Elementcount: u64 = 8;
const Root: u64 = 0;

pub enum MetaAddress {
    Root = 0,
    Order = 16,
    Elementcount = 8,
    FreeAdrr = 24,
}

#[derive(Debug)]
pub struct Bstar<T> {
    pub root: u64,
    pub elementcount: u64,
    pub order: u64,
    pub freeaddr: u64,
    pub eof: u64,
    meta: File,
    dat: File,
    type_save: PhantomData<T>,
}

impl<T: KnownSize + PartialOrd> Bstar<T> {
    pub fn delete(name: &str) -> Result<()> {
        try!(fs::remove_file(format!("{}.{}", name, "bsdat")));
        try!(fs::remove_file(format!("{}.{}", name, "bsmet")));
        Ok(())
    }

    pub fn load(name: &str) -> Result<Bstar<T>>{

        let mut _file = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(format!("{}.{}", name, "bsdat"));

        let mut dat = match _file {
            Ok(f) => f,
            Err(err) => return Err(err),
        };
        
        _file = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(format!("{}.{}", name, "bsmet"));

        let mut meta = match _file {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        try!(meta.seek(SeekFrom::Start(0)));

        let root = try!(meta.read_u64::<BigEndian>());
        meta.seek(SeekFrom::Current(8));
        let elementcount = try!(meta.read_u64::<BigEndian>());
        meta.seek(SeekFrom::Current(8));
        let order = try!(meta.read_u64::<BigEndian>());
        meta.seek(SeekFrom::Current(8));
        let free_addr = try!(meta.read_u64::<BigEndian>());
        meta.seek(SeekFrom::Current(8));
        let eof = try!(meta.read_u64::<BigEndian>());
        Ok(Bstar {
                root: root,
                order: order,
                elementcount: elementcount,
                freeaddr: free_addr,
                eof: eof,
                meta: meta,
                dat: dat,
                type_save: PhantomData
            }
        )

    }



    pub fn create(name: &str, order: u64) -> Result<Bstar<T>> {
        let mut _file = OpenOptions::new()
                                    .read(true)
                                    .write(true)
                                    .create(true)
                                    .open(format!("{}.{}", name, "bsdat"));

        let mut dat = match _file {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        _file = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(format!("{}.{}", name, "bsmet"));

        let mut meta = match _file {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        try!(meta.seek(SeekFrom::Start(0)));

        // IMPORTANT: Update the root start when changing B-tree fields!
        try!(meta.write_u64::<BigEndian>(0));
        meta.seek(SeekFrom::Current(8));
        // Write Elementcount
        try!(meta.write_u64::<BigEndian>(0));
        meta.seek(SeekFrom::Current(8));
        // Order meta
        try!(meta.write_u64::<BigEndian>(order));
        meta.seek(SeekFrom::Current(8));
        // Write first free address meta
        try!(meta.write_u64::<BigEndian>(0));
        meta.seek(SeekFrom::Current(8));
        // Write eof
        try!(meta.write_u64::<BigEndian>(0));

        Ok(Bstar {
                root: 0,
                order: order,
                elementcount: 0,
                freeaddr: 0,
                eof:0,
                meta: meta,
                dat: dat,
                type_save: PhantomData,
            }
        )
    }



    pub fn get_root(&mut self) -> Result<Bnode<T>> {
        Ok(try!(Bnode::read(&mut self.dat, Some(self.root))))
    }


    pub fn insert_keyaddr(&mut self, key: KeyAddr<T>) -> Result<Bnode<T>> {
        let lookup = try!(self.lookup_internal(&key));
        if lookup.bnode.is_some() {

            if lookup.found {
                panic!("Key already inserted!");
            } else {

                let mut node = lookup.bnode.unwrap();

                if node.node_list.elementcount == self.order * 2 {
                    // Node Overflow: Splitup and generate new father

                    panic!("Unimplemented: too many inserts!!");
                } else {
                    // Normal Insert
                    node.node_list.insert(key);
                    try!(node.write(&mut self.dat, Some(lookup.addr)));
                    try!(self.inc_elementcount());
                    Ok(node)
                }
            }



        } else {
            // if tree is empty create new root node
            try!(self.dat.seek(SeekFrom::Start(lookup.addr)));
            let mut list = SortedList::<KeyAddr<T>>::with_capacity((self.order * 2) as usize);
            list.insert(key);
            let mut node = Bnode::create(list, 0, 1, self.order);
            try!(node.write(&mut self.dat, Some(lookup.addr)));
            try!(self.inc_elementcount());
            Ok(node)
        }
    }

    fn inc_elementcount(&mut self) -> Result<()> {
        self.elementcount += 1;
        try!(self.meta.seek(SeekFrom::Start(Elementcount)));
        try!(self.meta.write_u64::<BigEndian>(self.elementcount));
        Ok(())
    }

    // returns the 
    fn lookup_internal(&mut self, key: &KeyAddr<T>) -> Result<InternalLookup<T>> {
        if self.elementcount == 0 {
            Ok(InternalLookup {
                found: false,
                bnode: None,
                addr: try!(self.use_free_addr()),
                index: None,
                target: None} )

        } else {
            let mut addr = self.root;
            let mut node = try!(Bnode::<T>::read(& mut self.dat, Some(addr)));
            let mut res = node.node_list.get_index_by_key(key);
            while node.is_leaf == 0 {
                addr = node.node_list.get_by_index(res.1).unwrap().addr;
                node = try!(Bnode::<T>::read(&mut self.dat, Some(addr)));
                res = node.node_list.get_index_by_key(key);
            }

            if res.0 {
                // if key was found
                let target = node.node_list.get_by_index(res.1).unwrap().addr;
                Ok(InternalLookup {
                    found: true,
                    bnode: Some(node),
                    addr: addr,
                    index: Some(res.1 as u64) ,
                    target: Some(target),
                })
            } else {
                Ok(InternalLookup {
                    found: false,
                    bnode: Some(node),
                    addr: addr,
                    index: Some(res.1 as u64) ,
                    target: None,
                })

            }
            
        }
    }

    // uses the next free address and updates meta data
    // USE ONLY IF INSTERTING A NEW NODE TO THE FREE ADDR!!!
    fn use_free_addr(&mut self) -> Result<u64> {
        if self.freeaddr != self.eof {
            try!(self.dat.seek(SeekFrom::Start(self.freeaddr)));
            let next_free = try!(self.dat.read_u64::<BigEndian>());
            try!(self.meta.seek(SeekFrom::Start(FreeAdrr)));
            try!(self.meta.write_u64::<BigEndian>(next_free));
            let tmp = self.freeaddr;
            self.freeaddr = next_free; 
            Ok(tmp)
        } else {
            let tmp = self.freeaddr;
            self.freeaddr += Bnode::<T>::size(self.order);
            self.eof = self.freeaddr;
            try!(self.meta.seek(SeekFrom::Start(FreeAdrr)));
            try!(self.meta.write_u64::<BigEndian>(self.freeaddr));
            Ok(tmp)
        }
    }

    // Idea: next Free Address is stored in .meta
    // If a node is deleted, free address in meta is updated to
    // the nodes address and the node space is used to store a pointer to
    // the last free address.
    // Importend!!!!!!!!! THIS WILL MAKE THE NODE AT addr INVALID!!
    // ONLY USE AFTER DELETING THE NODE AT addr!!!!!!!!!!!
    fn update_free_addr(&mut self, addr: u64) -> Result<()>{
        try!(self.meta.seek(SeekFrom::Start(FreeAdrr)));
        try!(self.dat.seek(SeekFrom::Start(addr)));
        try!(self.dat.write_u64::<BigEndian>(self.freeaddr));
        try!(self.meta.write_u64::<BigEndian>(addr));
        self.freeaddr = addr;
        Ok(())
    }

}

struct InternalLookup<T: PartialOrd + KnownSize> {
    // true if lookup found the KeyAddr
    found: bool,
    // the Node where the KeyAddr is to be located. If Tree is empty, bnode is None
    bnode: Option<Bnode<T>>,
    // the address of the Node in the BStar File
    addr: u64,
    // the index where KeyAddr is to be located in the SortedList of bnode
    // if tree is empty, index is None  
    index: Option<u64>,
    // the address targeting the datarecord in the table file
    target: Option<u64>
}



#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct Bnode<T: PartialOrd + KnownSize> {
    pub node_list: SortedList<KeyAddr<T>>,
    pub father: u64,
    // 0 = no leaf, else leaf
    pub is_leaf: u8,
    order: u64
}

impl<T: PartialOrd + KnownSize> Bnode<T> {

    pub fn create(node_list: SortedList<KeyAddr<T>>, father: u64, is_leaf: u8, order: u64) -> Bnode<T> {
        Bnode {
            node_list: node_list,
            father: father,
            is_leaf: is_leaf,
            order: order
        }
    }

    pub fn read(file: &mut File, addr: Option<u64>) -> Result<Bnode<T>> {
        try!(seek_maybe(file, addr));
        let father = try!(file.read_u64::<BigEndian>());
        file.seek(SeekFrom::Current(8));
        let is_leaf = try!(file.read_u8());
        file.seek(SeekFrom::Current(1));
        let elementcount = try!(file.read_u64::<BigEndian>());
        file.seek(SeekFrom::Current(8));
        let order = try!(file.read_u64::<BigEndian>());
        file.seek(SeekFrom::Current(8));

        let mut list = SortedList::<KeyAddr<T>>::with_capacity((order * 2) as usize);
        for i in 0..elementcount {
            let keyaddr = try!(KeyAddr::<T>::read(file, None ));
            list.insert(keyaddr);
            file.seek(SeekFrom::Current(KeyAddr::<T>::size() as i64));
        }

        Ok(Bnode { node_list: list, father: father, is_leaf: is_leaf, order: order } )
    }


    pub fn write(&mut self, file: &mut File, addr: Option<u64>) -> Result<()> {
        try!(seek_maybe(file, addr));
        try!(file.write_u64::<BigEndian>(self.father));
        file.seek(SeekFrom::Current(8));
        try!(file.write_u8(self.is_leaf));
        file.seek(SeekFrom::Current(1));
        try!(file.write_u64::<BigEndian>(self.node_list.elementcount));
        file.seek(SeekFrom::Current(8));

        try!(file.write_u64::<BigEndian>(self.order));
        file.seek(SeekFrom::Current(8));
        for i in 0..self.order * 2 {
            match self.node_list.get_by_index(i as usize) {
                Some(keyaddr) => {
                    try!(keyaddr.write(file, None))
                },
                None => try!(KeyAddr::<T>::write_default(file, None)),
            }
            file.seek(SeekFrom::Current(KeyAddr::<T>::size() as i64));
        }
        Ok(())
        
    }

    pub fn size(order: u64) -> u64 {
        KeyAddr::<T>::size()* order + 17
    }
}


#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct SortedList<T: PartialOrd> {
    pub list: Vec<T>,
    pub elementcount: u64,

}

impl<T: PartialOrd> SortedList<T> {

    pub fn new() -> SortedList<T> {
        SortedList { list: Vec::new(), elementcount: 0}
    }

    pub fn with_capacity(size: usize) -> SortedList<T> {
        SortedList { list: Vec::with_capacity(size), elementcount: 0 }
    }

    pub fn empty(&self) -> bool {
        self.elementcount == 0
    }

    pub fn insert(&mut self, value: T) -> bool {
        if self.empty() {
            self.list.push(value);
            self.elementcount +=1;
            true
        } else {
            let res = self.get_index_by_key_rec(&value, 0, (self.elementcount - 1) as usize);
            if self.list[res.1].partial_cmp(&value) == Some(Ordering::Less) {
                if res.1 == self.elementcount as usize {
                    self.list.push(value);
                } else {
                    self.list.insert(res.1 + 1, value);
                }
            } else {
                self.list.insert(res.1, value);
            }
            self.elementcount +=1;
            true
        }
    }

    pub fn split_at_index(&mut self, index: u64) {

    }

    pub fn split_at_key(&mut self, key: T) {

    }

    pub fn delete(&mut self, value: &T) -> Option<T> {
        if self.empty() {
            return None
        };
        let res = self.get_index_by_key_rec(value, 0, (self.elementcount - 1) as usize);
        if !res.0 {
            None
        } else {
            self.elementcount -=1;
            Some(self.list.remove(res.1))

        }
    }

    pub fn get_by_index(&mut self, index: usize) -> Option<&mut T> {
        if index >= 0 && index <= ( self.elementcount -1 ) as usize {
            Some(&mut self.list[index])
        } else {
            None
        }
    }


    pub fn get_by_key(&mut self, tofind: &T) -> Option<&mut T> {
        let res = self.get_index_by_key_rec(tofind, 0 , (self.elementcount - 1) as usize);
        if res.0 {
            Some(&mut self.list[res.1])
        } else {
            None
        }
    }

    pub fn get_index_by_key(&self, tofind: &T) -> (bool, usize) {
        self.get_index_by_key_rec(tofind, 0, (self.elementcount - 1) as usize)
    }

    fn get_index_by_key_rec(&self, tofind: &T, lo: usize, hi: usize) -> (bool, usize) {

        if hi == lo {
            if self.list[hi].partial_cmp(tofind) == Some(Ordering::Equal) {
                return (true, hi)
            }
            return (false, hi)
        } else if hi < lo {
            return (false, ( hi + lo ) /2)
        }

        let mid = (lo + hi + 1) / 2;

        match self.list[mid].partial_cmp(&tofind) {
            Some(Ordering::Equal) => (true, mid),
            Some(Ordering::Greater) => self.get_index_by_key_rec(tofind, lo, mid - 1),
            Some(Ordering::Less) => self.get_index_by_key_rec(tofind, mid + 1, hi),
            None => (false, mid)
        }
    }

}



#[derive(Debug,RustcDecodable, RustcEncodable, Clone)]
pub struct KeyAddr<T: PartialOrd + KnownSize> {
    pub key: T,
    pub addr: u64,
}

impl<T: PartialOrd + KnownSize> KeyAddr<T> {
    pub fn new(key: T, addr: u64) -> KeyAddr<T> {
        KeyAddr { key: key, addr: addr}
    }
}

impl<T: PartialOrd + KnownSize> PartialOrd for KeyAddr<T> {
    fn partial_cmp(&self, other:&Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T: PartialOrd + KnownSize> PartialEq for KeyAddr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}


impl<T: KnownSize + PartialOrd> KnownSize for KeyAddr<T> {
    fn size() -> u64 {
        // Size of Key + 4 for addr
        T::size() + 4
    }

    fn read(file: &mut File, addr: Option<u64>) -> Result<KeyAddr<T>> {
        let key = try!(T::read(file, addr));
        let tmp = try!(u64::read(file, None));
        Ok(KeyAddr::new(key,tmp))
    }

    fn write(&self, file: &mut File, addr: Option<u64>) -> Result<()> {
        try!(self.key.write(file, addr));
        Ok(try!(self.addr.write(file, None)))
    }

    fn write_default(file: &mut File, addr: Option<u64>) -> Result<()> {
        try!(T::write_default(file, addr));
        Ok(try!(u64::write_default(file, None)))
    }


}


impl KnownSize for u64 {
    fn size() -> u64 {
        8
    }

    fn read(file: &mut File, addr: Option<u64>) -> Result<u64> {
        try!(seek_maybe(file, addr));
        Ok(try!(file.read_u64::<BigEndian>()))
    }

    fn write(&self, file: &mut File, addr: Option<u64>) -> Result<()> {
        try!(seek_maybe(file, addr));
        Ok(try!(file.write_u64::<BigEndian>(*self)))
    }

    fn write_default(file: &mut File, addr: Option<u64>) -> Result<()> {
        try!(seek_maybe(file, addr));
        Ok(try!(file.write_u64::<BigEndian>(0)))
    }
}


fn seek_maybe(file: &mut File, addr: Option<u64>) -> Result<()> {
        Ok(match addr {
            Some(addr) => {
                try!(file.seek(SeekFrom::Start(addr)));
                ()
            },
            None => (),
        })

}


