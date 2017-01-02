use {Result, Error};
use file::{Header, Item, HEADER_SIZE, ITEM_SIZE};

use std::io::{Read, Write, Seek, SeekFrom};
use std::os::unix::io::AsRawFd;
use std::fs::{File, OpenOptions, remove_file};
use std::path::Path;
use nix::fcntl::{flock, FlockArg};

const HEADER_POS: u64 = 0;
const DEFAULT_KEY_COUNT: u32 = 256;

struct FoundItem {
    item: Item,
    pos: u64,
}

/// FileHashMap is a HashMap backed by a file.
pub struct FileHashMap {
    filename: &'static str,
}

impl FileHashMap {
    pub fn new(filename: &'static str) -> FileHashMap {
        FileHashMap {
            filename: filename,
        }
    }

    fn hash(s: &str, key_count: u32) -> u64 {
        let mut total = 0;
        for b in s.as_bytes() {
            total = (*b as u32).wrapping_add(total << 6).wrapping_add(total << 16).wrapping_sub(total);
        }
        (total % key_count) as u64
    }

    fn init_file_once(&self, key_count: u32) -> Result<()> {
        let path = Path::new(self.filename);
        let header = Header {
            version: 0,
            key_count: key_count,
            val_size: ITEM_SIZE,
            heap_size: 0,
        };

        // try to create the file, return OK if it already exists
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Err(_) => return Ok(()),
            Ok(f) => f,
        };

        // write header
        let buf = header.as_bytes();
        if let Err(_) = file.write(&buf) {
            return Err(Error::IO);
        }

        // write body
        for _ in 0..header.key_count {
            let buf = [0u8; ITEM_SIZE as usize];
            if let Err(_) = file.write(&buf) {
                return Err(Error::IO);
            }
        }
        Ok(())
    }

    /// removes the hashmap file for testing
    #[allow(unused_must_use, dead_code)]
    fn delete_file(&self) {
        let path = Path::new(self.filename);
        remove_file(&path);
    }

    fn seek_from(pos: u64) -> SeekFrom {
        SeekFrom::Start((pos * ITEM_SIZE as u64) + HEADER_SIZE as u64)
    }

    fn open_file(&self) -> File {
        let path = Path::new(self.filename);
        match OpenOptions::new().read(true).write(true).open(&path) {
            Err(why) => panic!("couldn't load {}: {:?}", path.display(), why),
            Ok(file) => file,
        }
    }

    fn read_header(&self, mut file: &File) -> Result<Header> {
        let hdr_s = SeekFrom::Start(HEADER_POS);
        if let Err(_) = file.seek(hdr_s) {
            return Err(Error::IO);
        }

        let mut buf = [0u8; HEADER_SIZE];
        if let Err(_) = file.read(&mut buf) {
            return Err(Error::IO);
        }

        Ok(Header::from(buf))
    }

    fn write_header(&self, mut file: &File, header: Header) -> Result<()> {
        let hdr_s = SeekFrom::Start(HEADER_POS);
        if let Err(_) = file.seek(hdr_s) {
            return Err(Error::IO);
        }

        let buf = header.as_bytes();
        if let Err(_) = file.write(&buf) {
            return Err(Error::IO);
        }
        Ok(())
    }

    fn read_item(&self, mut file: &File, pos: u64) -> Result<Item> {
        let s = FileHashMap::seek_from(pos);
        if let Err(_) = file.seek(s) {
            return Err(Error::IO);
        }

        let mut buf = [0u8; ITEM_SIZE as usize];
        if let Err(_) = file.read(&mut buf) {
            return Err(Error::IO);
        }
        Ok(Item::from(buf))
    }

    fn find_item(&self, file: &File, key: &str) -> Result<FoundItem> {
        if let Err(why) = self.init_file_once(DEFAULT_KEY_COUNT) {
            return Err(why);
        };
        let header = match self.read_header(&file) {
            Ok(x) => x,
            Err(why) => return Err(why),
        };
        let mut pos = FileHashMap::hash(key, header.key_count);
        loop {
            let item = match self.read_item(&file, pos) {
                Err(why) => return Err(why),
                Ok(x) => x,
            };

            // return key if found
            if item.is_key(key) {
                return Ok(FoundItem{
                    item: item,
                    pos: pos,
                });
            }

            // otherwise look for it in next item
            match item.get_next() {
                Some(x) => pos = x as u64,
                None    => return Err(Error::NotFound),
            }
        }
    }

    pub fn get(&self, key: &str) -> Result<String> {
        let file = self.open_file();
        match self.find_item(&file, key) {
            Err(why) => Err(why),
            Ok(f) => Ok(f.item.get_val()),
        }
    }

    pub fn remove(&self, key: &str) -> Result<()> {
        let file = self.open_file();
        loop {
            let found = match self.find_item(&file, key) {
                Err(why) => return Err(why),
                Ok(x) => x,
            };
            // overwrite with an empty item
            let new_item = Item::empty();
            // preserve its "next" value
            let merge_new_item = new_item.with_next(found.item.get_next());
            match self.write_item(&file, found.pos, &found.item, &merge_new_item) {
                Err(why) => return Err(why),
                Ok(ok) => if ok { break },
            }
        }
        Ok(())
    }

    fn write_item_no_lock(&self, mut file: &File, pos: u64, prev_item: &Item, new_item: &Item) -> Result<bool> {
        // read current contents and confirm nothing has changed
        let prev_item_confirm = match self.read_item(&file, pos) {
            Err(why) => return Err(why),
            Ok(x) => x,
        };
        if prev_item_confirm != *prev_item {
            return Ok(false);
        }

        // write new contents
        let s = FileHashMap::seek_from(pos);
        if let Err(_) = file.seek(s) {
            return Err(Error::IO);
        }
        let buf = new_item.as_bytes();
        if let Err(_) = file.write(&buf) {
            return Err(Error::IO);
        }
        return Ok(true);
    }

    fn write_item(&self, file: &File, pos: u64, prev_item: &Item, new_item: &Item) -> Result<bool> {
        // acquire an exclusive file lock
        let fd = file.as_raw_fd();
        flock(fd, FlockArg::LockExclusive).unwrap();

        // write the item
        let res = self.write_item_no_lock(file, pos, prev_item, new_item);

        // release lock
        flock(fd, FlockArg::Unlock).unwrap();
        return res;
    }

    fn write_new_item_to_heap_no_lock(&self, mut file: &File, prev_pos: u64, prev_item: &Item, new_item: &Item) -> Result<bool> {
        let mut header = match self.read_header(&file) {
            Ok(x) => x,
            Err(why) => return Err(why),
        };

        // calculate position of the new item and write it
        let new_pos = header.key_count + header.heap_size;
        let new_s = FileHashMap::seek_from(new_pos as u64);
        if let Err(_) = file.seek(new_s) {
            return Err(Error::IO);
        }
        let buf = new_item.as_bytes();
        if let Err(_) = file.write(&buf) {
            return Err(Error::IO);
        }

        // update prev item
        let update_prev_item = prev_item.with_next(Some(new_pos));
        if let Err(why) = self.write_item_no_lock(file, prev_pos, prev_item, &update_prev_item) {
            return Err(why);
        }

        // update header
        header.inc_heap();
        if let Err(why) = self.write_header(&file, header) {
            return Err(why);
        }
        return Ok(true);
    }

    fn write_new_item_to_heap(&self, file: &File, prev_pos: u64, prev_item: &Item, new_item: &Item) -> Result<bool> {
        // acquire an exclusive file lock
        let fd = file.as_raw_fd();
        flock(fd, FlockArg::LockExclusive).unwrap();

        // write the item
        let res = self.write_new_item_to_heap_no_lock(file, prev_pos, prev_item, new_item);

        // release lock
        flock(fd, FlockArg::Unlock).unwrap();
        return res;
    }

    pub fn insert(&self, key: &str, val: &str) -> Result<()> {
        if let Err(why) = self.init_file_once(DEFAULT_KEY_COUNT) {
            return Err(why);
        };
        let file = self.open_file();
        let header = match self.read_header(&file) {
            Ok(x) => x,
            Err(why) => return Err(why),
        };
        let mut pos = FileHashMap::hash(key, header.key_count);
        let new_item = Item::new(key, val);

        loop {
            let item = match self.read_item(&file, pos) {
                Err(why) => return Err(why),
                Ok(x) => x,
            };

            // write new item into static allocation
            if item.is_empty() {
                if let Ok(ok) = self.write_item(&file, pos, &item, &new_item) {
                    if !ok { continue; }
                }
                return Ok(());
            }

            // update item if already exists
            if item.is_key(key) {
                // preserve its "next" value
                let merge_new_item = new_item.with_next(item.get_next());
                if let Ok(ok) = self.write_item(&file, pos, &item, &merge_new_item) {
                    if !ok { continue; }
                }
                return Ok(());
            }

            // otherwise look for it in next item
            match item.get_next() {
                Some(x) => {
                    pos = x as u64;
                },
                None => {
                    if let Ok(ok) = self.write_new_item_to_heap(&file, pos, &item, &new_item) {
                        if !ok { continue; }
                    }
                    return Ok(());
                },
            }
        }
    }
}


#[test]
fn test_hash() {
    let n = 256u32;
    let key1 = String::from("foo");
    let key2 = String::from("bar");
    let h1 = FileHashMap::hash(key1.as_str(), n);
    let h2 = FileHashMap::hash(key1.as_str(), n);
    let h3 = FileHashMap::hash(key2.as_str(), n);
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[test]
fn test_filemap() {
    // create a file hashmap with 1 bucket to ensure collisions
    let fm = FileHashMap::new("test.kvlite");
    fm.delete_file();
    fm.init_file_once(1).unwrap();

    // get nonexistent key
    let val = fm.get("foo");
    assert_eq!(val.unwrap_err(), Error::NotFound); 

    // create key
    fm.insert("foo", "bar").unwrap();
    let val = fm.get("foo");
    assert_eq!(val.unwrap(), "bar");

    // update key
    fm.insert("foo", "baz").unwrap();
    let val = fm.get("foo");
    assert_eq!(val.unwrap(), "baz");

    // add second key and check it
    fm.insert("doo", "dah").unwrap();
    let val = fm.get("doo");
    assert_eq!(val.unwrap(), "dah");

    // add third key and check it
    fm.insert("uma", "duma").unwrap();
    let val = fm.get("uma");
    assert_eq!(val.unwrap(), "duma");

    // make sure old keys are still there
    let val = fm.get("foo");
    assert_eq!(val.unwrap(), "baz");
    let val = fm.get("doo");
    assert_eq!(val.unwrap(), "dah");

    // delete a key and check it
    fm.remove("foo").unwrap();
    let val = fm.get("foo");
    assert_eq!(val.unwrap_err(), Error::NotFound); 

    // make sure old keys are still there
    let val = fm.get("uma");
    assert_eq!(val.unwrap(), "duma");
    let val = fm.get("doo");
    assert_eq!(val.unwrap(), "dah");
}
