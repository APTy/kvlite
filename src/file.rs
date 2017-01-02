use std::mem;

pub const ITEM_SIZE: u32 = 1284;
pub const HEADER_SIZE: usize = 16;

/// Header is the first 16 bytes of a filemap.
#[derive(Debug)]
pub struct Header {
    pub version: u32,
    pub key_count: u32,
    pub val_size: u32,
    pub heap_size: u32,
}

impl Header {
    pub fn from(b: [u8; HEADER_SIZE]) -> Header {
        unsafe {
            Header {
                version: mem::transmute([b[0], b[1], b[2], b[3]]),
                key_count: mem::transmute([b[4], b[5], b[6], b[7]]),
                val_size: mem::transmute([b[8], b[9], b[10], b[11]]),
                heap_size: mem::transmute([b[12], b[13], b[14], b[15]]),
            }
        }
    }

    pub fn as_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut b = [0u8; HEADER_SIZE];
        unsafe {
            let version = mem::transmute::<u32, [u8; 4]>(self.version);
            let key_count = mem::transmute::<u32, [u8; 4]>(self.key_count);
            let val_size = mem::transmute::<u32, [u8; 4]>(self.val_size);
            let heap_size = mem::transmute::<u32, [u8; 4]>(self.heap_size);
            for i in 0..b.len() {
                match i {
                    0 ... 3   => { b[i] = version[i] },
                    4 ... 7   => { b[i] = key_count[i - 4] },
                    8 ... 11  => { b[i] = val_size[i - 8] },
                    12 ... 15 => { b[i] = heap_size[i - 12] },
                    _         => {},
                }
            }
        }
        b
    }

    pub fn inc_heap(&mut self) {
        self.heap_size += 1;
    }
}

/// Item is a linked-list node to store key-value pairs.
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Item {
    key: String,
    value: String,
    next: u32,
}

impl Item {
    pub fn new(key: &str, value: &str) -> Item {
        Item {
            key: String::from(key),
            value: String::from(value),
            next: 0u32,
        }
    }

    pub fn empty() -> Item {
        Item {
            key: String::from(""),
            value: String::from(""),
            next: 0u32,
        }
    }

    pub fn from(bytes: [u8; ITEM_SIZE as usize]) -> Item {
        let key_len = Item::find_null_byte(&bytes[0 .. 256]);
        let value_len = Item::find_null_byte(&bytes[256 .. 1280]);
        let key = String::from_utf8_lossy(&bytes[0 .. key_len]).into_owned();
        let value = String::from_utf8_lossy(&bytes[256 .. 256 + value_len]).into_owned();
        let next: u32;
        unsafe {
            next = mem::transmute::<[u8; 4], u32>([bytes[1280],bytes[1281],bytes[1282],bytes[1283]]);
        }
        Item {
            key: key,
            value: value,
            next: next,
        }
    }

    fn find_null_byte(bytes: &[u8]) -> usize {
        for i in 0..bytes.len() {
            if bytes[i] == 0u8 {
                return i;
            }
        }
        bytes.len()
    }

    pub fn as_bytes(&self) -> [u8; ITEM_SIZE as usize] {
        let mut b = [0u8; ITEM_SIZE as usize];
        let key = self.key.as_bytes();
        let value = self.value.as_bytes();
        let next: [u8; 4];
        unsafe {
            next = mem::transmute(self.next);
        }
        for i in 0..b.len() {
            match i {
                0 ... 255 if i < key.len()             => { b[i] = key[i] },
                256 ... 1279 if i - 256 < value.len()  => { b[i] = value[i - 256] },
                1280 ... 1284 if i - 1280 < next.len() => { b[i] = next[i - 1280] },
                _                                      => {},
            }
        }
        b
    }

    pub fn is_empty(&self) -> bool {
        &self.key == "" && &self.value == "" && self.next == 0u32
    }

    pub fn get_next(&self) -> Option<u32> {
        if self.next == 0u32 {
            Option::None
        } else {
            Option::Some(self.next)
        }
    }

    pub fn is_key(&self, key: &str) -> bool {
        self.key == key
    }

    pub fn get_val(self) -> String {
        self.value
    }

    pub fn with_next(&self, next: Option<u32>) -> Item {
        let n = match next {
            Some(n) => n,
            None => 0,
        };
        Item {
            key: self.key.clone(),
            value: self.value.clone(),
            next: n,
        }
    }
}

#[test]
fn test_item() {
    let item = Item {
        key: String::from("foo"),
        value: String::from("bar"),
        next: 0u32,
    };
    let new_item = Item::from(item.as_bytes());

    assert_eq!(new_item.key, item.key);
    assert_eq!(new_item.value, item.value);
    assert_eq!(new_item.next, item.next);

    assert!(Item::empty().is_empty());
}

#[test]
fn test_header() {
    let h = Header {
        version: 1,
        key_count: 16,
        val_size: 1024,
        heap_size: 100,
    };
    let new_h = Header::from(h.as_bytes());

    assert_eq!(h.version, new_h.version);
    assert_eq!(h.key_count, new_h.key_count);
    assert_eq!(h.val_size, new_h.val_size);
    assert_eq!(h.heap_size, new_h.heap_size);
}

