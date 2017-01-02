use std::io;

use hashmap::FileHashMap;

/// Returned by Store with a useful message.
#[derive(Debug)]
pub struct KVResult {
    msg: String,
}

impl KVResult {
    /// Returns a new result with the given message.
    fn new(msg: String) -> KVResult {
        KVResult {
            msg: msg,
        }
    }

    /// Returns a printable string.
    pub fn display(self) -> String {
        self.msg
    }
}

/// A key-value store backed by a local file.
pub struct Store {
    kv: FileHashMap,
}

impl Store {
    /// Create a new store using the given file.
    pub fn new(filename: &'static str) -> Store {
        Store {
            kv: FileHashMap::new(filename),
        }
    }

    /// Creates a key with a value or updates an already existing key's value.
    pub fn set(&self, key: &String, value: &String) -> Result<KVResult, io::Error> {
        let res = match self.kv.insert(key, value) {
            Some(_) => { KVResult::new(format!("UPDATE {}", key)) },
            None => { KVResult::new(format!("CREATE {}", key)) },
        };
        Result::Ok(res)
    }

    /// Gets the current value of a key.
    pub fn get(&self, key: &String) -> Result<KVResult, io::Error> {
        let res = match self.kv.get(key) {
            Some(val) => { KVResult::new(format!("{}", val)) },
            None => { KVResult::new(format!("NOT EXISTS {}", key)) },
        };
        Result::Ok(res)
    }

    /// Removes a key and its value.
    pub fn del(&self, key: &String) -> Result<KVResult, io::Error> {
        let res = match self.kv.remove(key) {
            Some(val) => { KVResult::new(format!("DELETE {}", val)) },
            None => { KVResult::new(format!("NOT EXISTS {}", key)) },
        };
        Result::Ok(res)
    }

    /// No-op can be used to signify an unknown command.
    pub fn noop(self, cmd: &String) -> Result<KVResult, io::Error> {
        Result::Ok(KVResult::new(format!("UNKNOWN {}", cmd)))
    }

}

#[test]
fn test_store() {
    let kv = Store::new("/tmp/testfile.kvlite");
    let foo = String::from("foo");
    let bar = String::from("bar");

    let s = kv.set(&foo, &bar);
    assert!(s.is_ok());

    let g = kv.get(&foo);
    assert!(g.is_ok());
    assert_eq!(g.unwrap().display(), bar);

    let d = kv.del(&foo);
    assert!(d.is_ok());
}
