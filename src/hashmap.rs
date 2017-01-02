use std::io::{Read, Write, Seek, SeekFrom};
use std::fs::{File, OpenOptions, remove_file};
use std::path::Path;

use file::{Header, Item, HEADER_SIZE, ITEM_SIZE};

const HEADER_POS: u64 = 0;

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
            total += b.count_ones();
        }
        (total % key_count as u32) as u64
    }

    fn init_file_once(&self) {
        let path = Path::new(self.filename);
        let header = Header {
            version: 0,
            key_count: 1,
            val_size: ITEM_SIZE,
            heap_size: 0,
        };
        let file = OpenOptions::new().write(true).create_new(true).open(&path);
        if let Ok(mut f) = file {
            // write header
            let buf = header.as_bytes();
            f.write(&buf);

            // write body
            for i in 0..header.key_count {
                let buf = [0u8; ITEM_SIZE as usize];
                f.write(&buf);
            }
        }
    }

    /// removes the hashmap file
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

    fn read_header(&self, mut file: &File) -> Header {
        let hdr_s = SeekFrom::Start(HEADER_POS);
        file.seek(hdr_s);

        let mut buf = [0u8; HEADER_SIZE];
        file.read(&mut buf);

        Header::from(buf)
    }

    fn write_header(&self, mut file: &File, header: Header) {
        let hdr_s = SeekFrom::Start(HEADER_POS);
        file.seek(hdr_s);

        let buf = header.as_bytes();
        file.write(&buf);
    }

    fn read_item(&self, mut file: &File, pos: u64) -> Item {
        let s = FileHashMap::seek_from(pos);
        file.seek(s);

        let mut buf = [0u8; ITEM_SIZE as usize];
        file.read(&mut buf);

        Item::from(buf)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.init_file_once();
        let file = self.open_file();
        let header = self.read_header(&file);
        let mut pos = FileHashMap::hash(key, header.key_count);
        loop {
            let item = self.read_item(&file, pos);
            if item.is_key(key) {
                return Option::Some(item.get_val());
            }
            match item.get_next() {
                Some(x) => pos = x as u64,
                None    => return Option::None,
            }
        }
    }

    fn write_item(&self, mut file: &File, pos: u64, item: Item) {
        let s = FileHashMap::seek_from(pos);
        file.seek(s);

        let buf = item.as_bytes();
        file.write(&buf);
    }

    fn write_new_item_to_heap(&self, mut file: &File, prev_pos: u64, mut prev_item: Item, new_item: Item) {
        let mut header = self.read_header(&file);

        // calculate position of the new item and write it
        let new_pos = header.key_count + header.heap_size;
        let new_s = FileHashMap::seek_from(new_pos as u64);
        file.seek(new_s);
        let buf = new_item.as_bytes();
        file.write(&buf);

        // update old item and write it
        let old_s = FileHashMap::seek_from(prev_pos);
        file.seek(old_s);
        prev_item.set_next(new_pos);
        let buf = prev_item.as_bytes();
        file.write(&buf);

        // update header
        header.inc_heap();
        self.write_header(&file, header);
    }

    pub fn insert(&self, key: &str, val: &str) -> Option<String> {
        self.init_file_once();
        let file = self.open_file();
        let header = self.read_header(&file);
        let mut pos = FileHashMap::hash(key, header.key_count);
        let new_item = Item::new(key, val);

        loop {
            let item = self.read_item(&file, pos);

            // write new item into static allocation
            if item.is_empty() {
                self.write_item(&file, pos, new_item);
                return Option::None;
            }

            // update item if already exists
            if item.is_key(key) {
                self.write_item(&file, pos, new_item);
                return Option::Some(item.get_val());
            }

            // otherwise look for it in next item
            match item.get_next() {
                Some(x) => {
                    pos = x as u64;
                },
                None => {
                    self.write_new_item_to_heap(&file, pos, item, new_item);
                    return Option::None;
                },
            }
        }
    }

    pub fn remove(&self, key: &str) -> Option<String> {
        self.init_file_once();
        let file = self.open_file();
        let header = self.read_header(&file);
        let mut pos = FileHashMap::hash(key, header.key_count);
        let new_item = Item::empty();

        loop {
            let item = self.read_item(&file, pos);

            // write new item into static allocation
            if item.is_empty() {
                self.write_item(&file, pos, new_item);
                return Option::None;
            }

            // update item if already exists
            if item.is_key(key) {
                self.write_item(&file, pos, new_item);
                return Option::Some(item.get_key());
            }

            // otherwise look for it in next item
            match item.get_next() {
                Some(x) => pos = x as u64,
                None    => return Option::None,
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
    let fm = FileHashMap::new("test.kvlite");
    fm.delete_file();

    // get nonexistent key
    let val = fm.get("foo");
    assert!(val.is_none());

    // create key
    fm.insert("foo", "bar");
    let val = fm.get("foo");
    assert!(val.is_some());
    assert_eq!(val.unwrap(), "bar");

    // update key
    fm.insert("foo", "baz");
    let val = fm.get("foo");
    assert!(val.is_some());
    assert_eq!(val.unwrap(), "baz");

    // add second key and check it
    fm.insert("doo", "dah");
    let val = fm.get("doo");
    assert!(val.is_some());
    assert_eq!(val.unwrap(), "dah");

    // add third key and check it
    fm.insert("uma", "duma");
    let val = fm.get("uma");
    assert!(val.is_some());
    assert_eq!(val.unwrap(), "duma");

    // make sure old keys are still there
    let val = fm.get("foo");
    assert!(val.is_some());
    assert_eq!(val.unwrap(), "baz");
    let val = fm.get("doo");
    assert!(val.is_some());
    assert_eq!(val.unwrap(), "dah");

    // delete a key and check it
    fm.remove("foo");
    let val = fm.get("foo");
    assert!(val.is_none());
}
