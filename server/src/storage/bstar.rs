
use std::io::*;
use std::fs::File;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::fs::OpenOptions;
use std::cmp::Ord;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Bstar {
    pub root: u64,
    pub order: u64,
    src: File,
}

impl Bstar {

    pub fn load(path: &str) -> Result<Bstar>{

        let mut _file = OpenOptions::new().read(true).write(true).append(true).open(path);

        let mut file = match _file {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        try!(file.seek(SeekFrom::Start(0)));

        let root = try!(file.read_u64::<BigEndian>());
        let order = try!(file.read_u64::<BigEndian>());
        Ok(Bstar { root: root, order: order, src: file } )

    }



    pub fn create(path: &str, order: u64) -> Result<Bstar> {
        let mut _file = OpenOptions::new().read(true).write(true).append(true).create(true).open(path);

        let mut file = match _file {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        try!(file.seek(SeekFrom::Start(0)));

        // IMPORTANT: Update the root start when changing B-tree fields!
        try!(file.write_u64::<BigEndian>(16));
        try!(file.write_u64::<BigEndian>(order));

        Ok(Bstar { root: 16, order: order ,src: file } )
    }



    pub fn get_root(&mut self) -> Result<Bnode> {
        Ok(try!(Bnode::read(&mut self.src, self.order, self.root)))
    }


    pub fn insert(&mut self, key: u64, addr: u64 ) -> Result<Bnode> {
        try!(self.src.seek(SeekFrom::Start(self.root)));
        let mut list = SortedList::with_capacity((self.order * 2) as usize);
        list.insert(KeyAddr::new(key, addr));
        let mut node = Bnode { node_list: list, father: 0, is_leaf: 1 };
        try!(node.write(&mut self.src, self.order, self.root));
        Ok(node)
    }
}





#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct Bnode {
    pub node_list: SortedList<KeyAddr<u64>>,
    pub father: u64,
    // 0 = no leaf, else leaf
    pub is_leaf: u8,

}

impl Bnode {

    pub fn read(file: &mut File,order: u64, index: u64) -> Result<Bnode> {
        file.seek(SeekFrom::Start(index));
        let father = try!(file.read_u64::<BigEndian>());
        let is_leaf = try!(file.read_u8());
        let elementcount = try!(file.read_u64::<BigEndian>());

        let mut list = SortedList::with_capacity((order * 2) as usize);
        for i in 0..elementcount {
            let key = try!(file.read_u64::<BigEndian>());
            let addr = try!(file.read_u64::<BigEndian>());
            list.insert(KeyAddr::new(key, addr));

        }

        Ok(Bnode { node_list: list, father: father, is_leaf: is_leaf } )
    }


    pub fn write(&mut self, file: &mut File,order: u64, index: u64) -> Result<()> {
        file.seek(SeekFrom::Start(index));
        try!(file.write_u64::<BigEndian>(self.father));
        try!(file.write_u8(self.is_leaf));
        try!(file.write_u64::<BigEndian>(self.node_list.elementcount));
        for i in 0..order * 2 {
            if self.node_list.get_by_index(i as usize).is_some() {
                let keyaddr = self.node_list.get_by_index(i as usize).unwrap();
                println!("{:?}", keyaddr );
                try!(file.write_u64::<BigEndian>(keyaddr.key));
                try!(file.write_u64::<BigEndian>(keyaddr.addr));
            }
            try!(file.write_u64::<BigEndian>(0));
            try!(file.write_u64::<BigEndian>(0));
        }

        Ok(())
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


    pub fn get_by_key(&mut self, tofind: &T) -> Option<&mut T>{
        let res = self.get_index_by_key_rec(tofind, 0 , (self.elementcount - 1) as usize);
        if res.0 {
            Some(&mut self.list[res.1])
        } else {
            None
        }
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
pub struct KeyAddr<T: PartialOrd> {
    pub key: T,
    pub addr: u64,
}

impl<T: PartialOrd> KeyAddr<T> {
    pub fn new(key: T, addr: u64) -> KeyAddr<T> {
        KeyAddr { key: key, addr: addr}
    }
}

impl<T: PartialOrd> PartialOrd for KeyAddr<T> {
    fn partial_cmp(&self, other:&Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T: PartialOrd> PartialEq for KeyAddr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}
