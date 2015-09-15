
use std::io::*;
use std::fs::File;
use std::fs;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::fs::OpenOptions;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::fmt::Debug;

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

#[derive(PartialEq)]
pub enum Side {
    Right,
    Left
}
#[derive(Debug)]
pub struct Bstar<T: Debug + PartialOrd + KnownSize> {
    pub root: u64,
    pub elementcount: u64,
    pub order: u64,
    pub freeaddr: u64,
    pub eof: u64,
    meta: File,
    dat: File,
    type_save: PhantomData<T>,
}

impl<T: KnownSize + PartialOrd + Clone + Debug> Bstar<T> {
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
        //meta.seek(SeekFrom::Current(8));
        let elementcount = try!(meta.read_u64::<BigEndian>());
        //meta.seek(SeekFrom::Current(8));
        let order = try!(meta.read_u64::<BigEndian>());
        //meta.seek(SeekFrom::Current(8));
        let free_addr = try!(meta.read_u64::<BigEndian>());
        //meta.seek(SeekFrom::Current(8));
        let eof = try!(meta.read_u64::<BigEndian>());

        meta.seek(SeekFrom::Start(0));
        dat.seek(SeekFrom::Start(0));
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
        //meta.seek(SeekFrom::Current(8));
        // Write Elementcount
        try!(meta.write_u64::<BigEndian>(0));
        //meta.seek(SeekFrom::Current(8));
        // Order meta
        try!(meta.write_u64::<BigEndian>(order));
        //meta.seek(SeekFrom::Current(8));
        // Write first free address meta
        try!(meta.write_u64::<BigEndian>(0));
        //meta.seek(SeekFrom::Current(8));
        // Write eof
        try!(meta.write_u64::<BigEndian>(0));


        meta.seek(SeekFrom::Start(0));
        dat.seek(SeekFrom::Start(0));
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

    pub fn debug_print(& mut self) -> Result<()>{
        let root = self.root;
        Ok(try!(self.debug_print_rec(root,"")))
    }

    fn debug_print_rec(&mut self, addr: u64, delim: &str) -> Result<()> {
        let node = try!(Bnode::<T>::read(& mut self.dat, Some(addr)));
        print!("{}{}:  ",delim, addr);
        for key in &node.node_list.list {
            print!("{:?} => {:?} ;  ",key.key, key.addr);
        }
        println!("");
        if node.is_leaf != 1 {
            for key in node.node_list.list {
                try!(self.debug_print_rec(key.addr,&format!("{}{}",delim,"|----")));
            }
        }
        Ok(())
    }

    pub fn lookup_keyaddr(&mut self, key: T) -> Result<Option<KeyAddr<T>>> {
        let lookup = try!(self.lookup_internal(& KeyAddr::new(key.clone(),0)));
        if lookup.found {
            Ok(Some(KeyAddr::new(key, lookup.target.unwrap())))
        } else {
            Ok(None)
        }
    }


    pub fn insert_keyaddr(&mut self, key: KeyAddr<T>) -> Result<Bnode<T>> {
        let lookup = try!(self.lookup_internal(&key));

        if lookup.bnode.is_some() {

            if lookup.found {
                // Key already exists
                panic!("Key already inserted!");
            } else {
                // Key does yet not exist
                let mut originalnode = lookup.bnode.unwrap();

                if originalnode.node_list.elementcount == self.order * 2 {

                // Node Overflow: split up and generate new father
                originalnode.node_list.insert(key);

                try!(self.inc_elementcount());
                self.delegate_overflow_father(&mut originalnode, lookup.addr)

                } else {
                    // Normal Insert

                    if originalnode.node_list.insert(key) == 0 {
                        // key for reaching this node changed!
                        let oldkey = originalnode.node_list.get_by_index(1).unwrap().key.clone();
                        self.delegate_reaching_key(&mut originalnode, oldkey );
                    }

                    try!(originalnode.write(&mut self.dat, Some(lookup.addr)));

                    try!(self.inc_elementcount());

                    Ok(originalnode)
                }
            }



        } else {
            // if tree is empty create new root node
            try!(self.dat.seek(SeekFrom::Start(lookup.addr)));
            let mut list = SortedList::<KeyAddr<T>>::with_capacity((self.order * 2) as usize);
            list.insert(key);
            let mut node = Bnode::create(list, 0, 1, 1, self.order);
            try!(node.write(&mut self.dat, Some(lookup.addr)));

            try!(self.inc_elementcount());
            Ok(node)
        }
    }


    fn delegate_overflow_father(&mut self, node: &mut Bnode<T>, addr: u64) -> Result<Bnode<T>>{
        if node.node_list.elementcount <= self.order * 2 {
            try!(node.write(&mut self.dat, Some(addr)));
            Bnode::<T>::read(&mut self.dat, Some(addr))

        } else {
            let fatheraddr = node.father;
            let index = node.node_list.elementcount as usize / 2;
            let rightlist = node.node_list.split_by_index(index);
            // right son: father is at the address of the node found with lookup
            let mut rightson = Bnode::create(rightlist, addr, node.is_leaf, 0, self.order);
            // For the father: create left and right son keyaddr that need to be inserted.
            let leftkey = node.node_list.get_by_index(0).unwrap().key.clone();
            let rightkey = rightson.node_list.get_by_index(0).unwrap().key.clone();
            let rightaddr = try!(self.use_free_addr());
            let rightkeyaddr = KeyAddr::new(rightkey, rightaddr);

            if rightson.is_leaf != 1 {
                for key in &rightson.node_list.list {
                    try!(self.dat.seek(SeekFrom::Start(key.addr)));
                    self.dat.write_u64::<BigEndian>(rightaddr);
                }
            }

            let leftaddr = addr;
            if node.is_root == 1 {

                // original node was the root node
                let newrootaddr = try!(self.use_free_addr());
                let leftaddr = addr;
                let leftkeyaddr = KeyAddr::new(leftkey, leftaddr);


                // left son: father as rightson.
                // since the nodelist of original was changed already, original is the new left son

                node.father = newrootaddr;
                node.is_root = 0;
                rightson.father = newrootaddr;
                let mut node_list = SortedList::<KeyAddr<T>>::new();
                node_list.insert(leftkeyaddr);
                node_list.insert(rightkeyaddr);
                let mut newroot = Bnode::create(node_list, self.root , 0, 1, self.order);


                // update the new root position
                self.update_root(newrootaddr);
                // and write the new root to disc
                newroot.write(&mut self.dat, Some(newrootaddr));

                // update the lef son data
                node.write(&mut self.dat, Some(leftaddr));
                // TODO: think about effective way of rewriting the node!!!

                // write the right son to his new position
                rightson.write(&mut self.dat, Some(rightaddr));
                // increase elementcount of tree
                Ok(newroot)

                } else {
                    // inner node:
                    let mut father = try!(Bnode::<T>::read(&mut self.dat, Some(fatheraddr)));
                    // insert the new keyaddr for right son
                    father.node_list.insert(rightkeyaddr);

                    // update rightsons father address
                    rightson.father = fatheraddr;
                    // write rightson to disk
                    try!(rightson.write(&mut self.dat, Some(rightaddr)));

                    // update the original nodes data, no rewrite required! the only thing that
                    // changed is the elementcount, yet these changes are TODO implemented
                    node.write(&mut self.dat, Some(leftaddr));

                    // deligate possible problem to father node
                    self.delegate_overflow_father(&mut father, fatheraddr)

                }
        }
    }

    fn delegate_reaching_key(&mut self, node: &mut Bnode<T>, oldkey: T) -> Result<()>{
        if node.is_root != 1 {
            let keyofinterest = node.node_list.get_by_index(0).unwrap().key.clone();
            //let oldkeyofinterest = node.node_list.get_by_index(1).unwrap().key.clone();
            let mut father = try!(Bnode::<T>::read(&mut self.dat, Some(node.father)));
            //let newoldkey = father.node_list.get_by_index(1).unwrap().key.clone();
            let sonaddress = father.node_list.delete_by_key(&KeyAddr::new(oldkey.clone(), 0)).unwrap().addr;
            let keyaddr = KeyAddr::<T>::new(keyofinterest , sonaddress);
            if father.node_list.insert(keyaddr) == 0 {
                try!(father.write(&mut self.dat, Some(node.father)));
                self.delegate_reaching_key(&mut father, oldkey)
            } else {
                try!(father.write(&mut self.dat, Some(node.father)));
                Ok(())
            }

        } else {
            Ok(())
        }
    }


    pub fn delete_keyaddr(&mut self, key: T) -> Result<Option<KeyAddr<T>>> {
        if self.elementcount == 0 {
            Ok(None)
        } else {
            let lookup  = try!(self.lookup_internal(&KeyAddr::<T>::new(key.clone(), 0 )));
            if lookup.found {
                try!(self.delegate_underflow_node(&mut lookup.bnode.unwrap(), lookup.index.unwrap(), lookup.addr));
                try!(self.dec_elementcount());
                Ok(Some(KeyAddr::<T>::new(key, lookup.target.unwrap())))

            } else {
                Ok(None)
            }
        }
    }

    fn delegate_underflow_node(&mut self, node: &mut Bnode<T>, keyindex: u64, nodeaddr: u64) -> Result<()> {
            if node.node_list.elementcount > self.order {
                // normal delete
                let res = node.node_list.delete_by_index(keyindex as usize);
                if keyindex == 0 {
                    self.delegate_reaching_key(node, res.unwrap().key);
                }
                node.write(&mut self.dat, Some(nodeaddr));
                Ok(())

            } else {
                if node.is_root == 1 {
                    // delete from root
                    if node.node_list.elementcount <= 2 {
                        // delete old root
                        Ok(())
                    } else {
                        // normal
                        node.node_list.delete_by_index(keyindex as usize);
                        node.write(&mut self.dat, Some(nodeaddr));
                        Ok(())
                    }
                } else {
                    // Node does not have enough keys to delete from
                    let deleted = node.node_list.delete_by_index(keyindex as usize).unwrap();
                    if keyindex == 0 {
                        self.delegate_reaching_key(node, deleted.key.clone());
                    }
                    let mut father = try!(Bnode::<T>::read(&mut self.dat, Some(node.father)));
                    let indexonfather = father.node_list.get_index_by_key(&deleted).1;

                    // right brother
                    let rightbaddr = match father.node_list.get_by_index(indexonfather + 1) {
                        Some(keyaddr) => Some(keyaddr.addr),
                        None => None,
                    };
                    let leftbaddr = match indexonfather {
                        0 => None,
                        _ => Some(father.node_list.get_by_index(indexonfather - 1).unwrap().addr),
                    };

                    let mut peernode = {
                        if rightbaddr != None && leftbaddr != None {
                            // compare size ofbrother nodes
                            let rightbnode = try!(Bnode::<T>::read(&mut self.dat, Some(rightbaddr.unwrap())));
                            let leftbnode = try!(Bnode::<T>::read(&mut self.dat, Some(leftbaddr.unwrap())));
                            if rightbnode.node_list.elementcount > leftbnode.node_list.elementcount {
                                (rightbnode, Side::Right, rightbaddr.unwrap())
                            } else {
                                (leftbnode, Side::Left, leftbaddr.unwrap())
                            }
                        } else if rightbaddr == None {
                            // return left brother
                            (try!(Bnode::<T>::read(&mut self.dat, Some(leftbaddr.unwrap()))), Side::Left, leftbaddr.unwrap())
                        } else {
                            // return right brother
                            (try!(Bnode::<T>::read(&mut self.dat, Some(rightbaddr.unwrap()))), Side::Right, rightbaddr.unwrap())
                        }

                    };

                    if peernode.0.node_list.elementcount <= self.order {
                        // TODO: merge node and peernode





                    } else {
                        // distribute nodelists from both nodes equally

                        let peerlength = peernode.0.node_list.elementcount;
                        let mut nodelength = node.node_list.elementcount;

                        if peernode.1 == Side::Left {
                            // left node
                            for i in 0..((peerlength - nodelength)/2) {
                                let tmp = peernode.0.node_list.delete_by_index((peerlength-i-1)as usize);
                                node.node_list.insert_at_index(0, tmp.unwrap());
                            }

                            father.node_list.get_by_index(indexonfather).unwrap().key = node.node_list.get_by_index(0).unwrap().key.clone();
                            try!(peernode.0.write(&mut self.dat, Some(peernode.2)));


                        } else {
                            for i in 0..((peerlength - nodelength)/2) {
                                let tmp = peernode.0.node_list.delete_by_index(i as usize);
                                node.node_list.insert_at_index(nodelength as usize, tmp.unwrap());
                                let mut nodelength = node.node_list.elementcount;
                            }

                            father.node_list.get_by_index(indexonfather + 1).unwrap().key = peernode.0.node_list.get_by_index(0).unwrap().key.clone();
                            try!(peernode.0.write(&mut self.dat, Some(peernode.2)));
                        }
                        node.write(&mut self.dat, Some(nodeaddr));
                        // TODO: Make more effective disc usage: update the father only on the index where it is changed
                        father.write(&mut self.dat, Some(node.father));

                    }
                    // if two nodes are merged: delegate deleting the unneccessary key to father node.
                    Ok(())
                }

            }
    }


    fn inc_elementcount(&mut self) -> Result<()> {
        self.elementcount += 1;
        self.meta.seek(SeekFrom::Start(8));
        Ok(try!(self.meta.write_u64::<BigEndian>(self.elementcount)))
    }

    fn dec_elementcount(&mut self) -> Result<()> {
        self.elementcount -= 1;
        self.meta.seek(SeekFrom::Start(8));
        Ok(try!(self.meta.write_u64::<BigEndian>(self.elementcount)))
    }

    fn update_root(&mut self, root: u64) -> Result<()> {
        self.root = root;
        try!(self.meta.seek(SeekFrom::Start(Root)));
        Ok(try!(self.meta.write_u64::<BigEndian>(root)))
    }

    // returns the lookupinfo
    fn lookup_internal(&mut self, key: &KeyAddr<T>) -> Result<InternalLookup<T>> {
        if self.elementcount == 0 {
            // if tree is empty
            Ok(InternalLookup {
                found: false,
                bnode: None,
                addr: try!(self.use_free_addr()),
                index: None,
                target: None} )

        } else {
            // tree is not empty
            let mut addr = self.root;
            let mut node = try!(Bnode::<T>::read(& mut self.dat, Some(addr)));
            let mut res = node.node_list.get_index_by_key(key);

            // from the root starting search down to the leaf
            while node.is_leaf == 0 {
                let mut index = res.1;
                if node.node_list.list[index].gt(key) && index != 0 {
                    index -= 1;
                }

                addr = node.node_list.get_by_index(index).unwrap().addr;
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
                // key was not found
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
            try!(self.meta.write_u64::<BigEndian>(self.eof));
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

#[derive(Debug)]
struct InternalLookup<T: PartialOrd + KnownSize + Debug> {
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


const BnodeElementCountOffset: u64 = 10;
const BnodeIsRootOffset: u64 = 9;
#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct Bnode<T: PartialOrd + KnownSize + Debug> {
    pub node_list: SortedList<KeyAddr<T>>,
    pub father: u64,
    // 0 = no leaf, else leaf
    pub is_leaf: u8,
    //0 = no root, else root
    pub is_root: u8,
    order: u64
}

impl<T: PartialOrd + KnownSize + Debug> Bnode<T> {

    pub fn create(node_list: SortedList<KeyAddr<T>>, father: u64, is_leaf: u8, is_root: u8, order: u64) -> Bnode<T> {
        Bnode {
            node_list: node_list,
            father: father,
            is_leaf: is_leaf,
            is_root: is_root,
            order: order
        }
    }

    pub fn read(file: &mut File, addr: Option<u64>) -> Result<Bnode<T>> {
        try!(seek_maybe(file, addr));
        let father = try!(file.read_u64::<BigEndian>());
        let is_leaf = try!(file.read_u8());
        let is_root = try!(file.read_u8());
        let elementcount = try!(file.read_u64::<BigEndian>());
        let order = try!(file.read_u64::<BigEndian>());
        let mut list = SortedList::<KeyAddr<T>>::with_capacity((order * 2) as usize);
        for i in 0..elementcount {
            let keyaddr = try!(KeyAddr::<T>::read(file, None ));
            list.insert(keyaddr);
        }

        Ok(Bnode { node_list: list, father: father, is_leaf: is_leaf, is_root: is_root, order: order } )
    }


    pub fn write(&mut self, file: &mut File, addr: Option<u64>) -> Result<()> {
        try!(seek_maybe(file, addr));
        try!(file.write_u64::<BigEndian>(self.father));
        try!(file.write_u8(self.is_leaf));
        try!(file.write_u8(self.is_root));
        try!(file.write_u64::<BigEndian>(self.node_list.elementcount));
        try!(file.write_u64::<BigEndian>(self.order));
        for i in 0..self.order * 2 {
            match self.node_list.get_by_index(i as usize) {
                Some(keyaddr) => {
                    try!(keyaddr.write(file, None));
                },
                None => (),
            }
        }
        Ok(())

    }

    pub fn size(order: u64) -> u64 {
        ((KeyAddr::<T>::size() * (order * 2)) + 26)
    }
}


#[derive(Debug,RustcDecodable, RustcEncodable)]
pub struct SortedList<T: PartialOrd + Debug> {
    pub list: Vec<T>,
    pub elementcount: u64,

}

impl<T: PartialOrd + Debug> SortedList<T> {

    pub fn new() -> SortedList<T> {
        SortedList { list: Vec::new(), elementcount: 0}
    }

    pub fn with_capacity(size: usize) -> SortedList<T> {
        SortedList { list: Vec::with_capacity(size), elementcount: 0 }
    }

    pub fn empty(&self) -> bool {
        self.elementcount == 0
    }

    pub fn insert_at_index(&mut self, index: usize, key: T) {
        self.list.insert(index,key);
        self.elementcount += 1;
    }
    /// returns the index where the inserted value is located
    pub fn insert(&mut self, value: T) -> u64 {
        if self.empty() {
            self.list.push(value);
            self.elementcount +=1;
            0
        } else {
            let res = self.get_index_by_key_rec(&value, 0, (self.elementcount - 1) as usize);
            if self.list[res.1].partial_cmp(&value) == Some(Ordering::Less) {
                self.list.insert(res.1 + 1, value);
                self.elementcount +=1;
                (res.1 + 1) as u64
            } else {
                self.list.insert(res.1, value);
                self.elementcount +=1;
                res.1 as u64
            }
        }
    }

    /// Splits the SortedList into 2 based on index.
    /// After calling this function the original list will contain
    /// the data from [0, index], the returned List will contain the data from
    /// (index, elementcount)
    ///
    /// panics if index is out of bounds
    pub fn split_by_index(&mut self, index: usize) -> SortedList<T>{
        let mut second = SortedList::<T>::new();
        let tmp = self.elementcount;
        for i in 1..(tmp - (index as u64)) {
            second.list.insert(0, self.list.remove((tmp-i) as usize));
            second.elementcount+=1;
            self.elementcount-=1;
        }

        second

    }

    pub fn split_by_key(&mut self, key: &T) -> SortedList<T> {
        let index = self.get_index_by_key(&key).1;
        self.split_by_index(index)
    }

    pub fn delete_by_key(&mut self, value: &T) -> Option<T> {
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

    pub fn delete_by_index(&mut self, index: usize) -> Option<T> {
        if  index >= 0 && index <= ( self.elementcount -1 ) as usize {
            self.elementcount-=1;
            Some(self.list.remove(index))
        } else {
            None
        }
    }

    pub fn get_by_index(&mut self, index: usize) -> Option<&mut T> {
        if index >= 0 && index <= ( self.elementcount - 1 ) as usize {
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
pub struct KeyAddr<T: PartialOrd + KnownSize + Debug> {
    pub key: T,
    pub addr: u64,
}

impl<T: PartialOrd + KnownSize + Debug> KeyAddr<T> {
    pub fn new(key: T, addr: u64) -> KeyAddr<T> {
        KeyAddr { key: key, addr: addr}
    }
}

impl<T: PartialOrd + KnownSize + Debug> PartialOrd for KeyAddr<T> {
    fn partial_cmp(&self, other:&Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T: PartialOrd + KnownSize + Debug> PartialEq for KeyAddr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}


impl<T: KnownSize + PartialOrd + Debug> KnownSize for KeyAddr<T> {
    fn size() -> u64 {
        // Size of Key + 8 for addr
        T::size() + 8
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
        try!(seek_maybe(file, addr));
        try!(T::write_default(file, None));
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
