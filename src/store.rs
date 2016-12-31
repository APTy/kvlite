extern crate rustc_serialize;

use std::io;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::path::Path;
use std::error::Error;
use std::collections::HashMap;
use self::rustc_serialize::json;

/// Returned by Store with a useful message.
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
    filename: &'static str,
}

impl Store {
    /// Create a new store using the given file.
    pub fn new(filename: &'static str) -> Store {
        Store {
            filename: filename,
        }
    }

    /// Creates a key with a value or updates an already existing key's value.
    pub fn set(&self, key: &String, value: &String) -> Result<KVResult, io::Error> {
        let mut kv = self.load();
        let res = match kv.insert(key.clone(), value.clone()) {
            Some(_) => { KVResult::new(format!("UPDATE {}", key)) },
            None => { KVResult::new(format!("CREATE {}", key)) },
        };
        match self.commit(&kv) {
            Err(why) => { Result::Err(why) },
            _ => { Result::Ok(res) },
        }
    }

    /// Gets the current value of a key.
    pub fn get(&self, key: &String) -> Result<KVResult, io::Error> {
        let kv = self.load();
        let res = match kv.get(key) {
            Some(val) => { KVResult::new(format!("{}", val)) },
            None => { KVResult::new(format!("NOT EXISTS {}", key)) },
        };
        Result::Ok(res)
    }

    /// Removes a key and its value.
    pub fn del(&self, key: &String) -> Result<KVResult, io::Error> {
        let mut kv = self.load();
        let res = match kv.remove(key) {
            Some(val) => { KVResult::new(format!("DELETE {}", val)) },
            None => { KVResult::new(format!("NOT EXISTS {}", key)) },
        };
        match self.commit(&kv) {
            Err(why) => { Result::Err(why) },
            _ => { Result::Ok(res) },
        }
    }

    /// No-op can be used to signify an unknown command.
    pub fn noop(self, cmd: &String) -> Result<KVResult, io::Error> {
        Result::Ok(KVResult::new(format!("UNKNOWN {}", cmd)))
    }

    /// Reads the current database state into memory.
    fn load(&self) -> HashMap<String, String> {
        let path = Path::new(self.filename);
        let mut file = match OpenOptions::new().read(true).write(true).create(true).open(&path) {
            Err(why) => panic!("couldn't load {}: {:?}", path.display(), why),
            Ok(file) => file,
        };

        let mut s = String::new();
        let fr = file.read_to_string(&mut s);
        if fr.is_err() {
            panic!("couldn't read {:?}: {:?}", path, fr);
        }
        if s.len() == 0 {
            return HashMap::new();
        }

        let kv: HashMap<String, String> = match json::decode(&s) {
            Err(why) => panic!("couldn't parse: {}", why.description()),
            Ok(x) => x,
        };
        kv
    }

    /// Saves the in-memory database state to file storage.
    fn commit(&self, kv: &HashMap<String, String>) -> Result<(), io::Error> {
        let d = json::encode(&kv).unwrap();
        let path = Path::new(self.filename);
        let display = path.display();

        let mut file = match OpenOptions::new().write(true).create(true).open(&path) {
            Err(why) => panic!("couldn't save {}: {}", display, why.description()),
            Ok(file) => file,
        };

        let fw = file.write_all(d.as_bytes());
        if fw.is_err() {
            return fw;
        }
        let fl = file.set_len(d.len() as u64);
        if fl.is_err() {
            return fw;
        }
        file.sync_data()
    }
}
