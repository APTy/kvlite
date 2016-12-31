/// kvlite
extern crate rustc_serialize;

use std::io;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::env::args;
use std::path::Path;
use std::error::Error;
use std::collections::HashMap;
use rustc_serialize::json;

static DB_FILENAME: &'static str = "./db.kvlite";

struct KVStore {
    kv: HashMap<String, String>,
    filename: &'static str,
}

impl KVStore {
    fn new(filename: &'static str) -> KVStore {
        KVStore {
            kv: HashMap::new(),
            filename: filename,
        }
    }

    fn set(&mut self, key: &String, value: &String) -> Option<String> {
        self.kv.insert(key.clone(), value.clone())
    }

    fn get(&self, key: &String) -> Option<&String> {
        self.kv.get(key)
    }

    fn del(&mut self, key: &String) -> Option<String> {
        self.kv.remove(key)
    }

    fn load(&mut self) {
        let path = Path::new(self.filename);
    
        let mut file = match OpenOptions::new().read(true).write(true).create(true).open(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why.description()),
            Ok(file) => file,
        };
    
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => {
                panic!("couldn't read {}: {}", path.display(), why.description());
            },
            Ok(_) => {
                if s.len() == 0 {
                    return;
                }
                let kv: HashMap<String, String> = match json::decode(&s) {
                    Err(why) => panic!("couldn't parse: {}", why.description()),
                    Ok(x) => x,
                };
                for (key, val) in &kv {
                    self.set(key, val);
                }
            },
        };
    }

    fn commit(self) -> Result<(), io::Error> {
        let d = json::encode(&self.kv).unwrap();
        let path = Path::new(self.filename);
        let display = path.display();

        let mut file = match OpenOptions::new().write(true).create(true).open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
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

fn help() {
    println!("usage: kvlite <command> [<args>]\n");
    println!("kvlite is a key-value store backed by the local file system.\n");
    println!("commands:");
    println!("  set <key> <value>        Create or update a key's value.");
    println!("  get <key>                Look up a key's value.");
    println!("  del <key>                Remove a key.");
}

fn main() {
    let mut kv = KVStore::new(DB_FILENAME);
    kv.load();

    let args: Vec<String> = args().collect();
    if args.len() == 1 {
        help();
        return;
    }
    let cmd = &args[1];
    match cmd.as_str() {
        "set" => {
            if args.len() < 4 {
                help();
                return;
            }
            let key = &args[2];
            let val = &args[3];
            match kv.set(key, val) {
                Some(_) => {
                    println!("UPDATE {}", key);
                },
                None => {
                    println!("CREATE {}", key);
                },
            }
        },
        "get" => {
            if args.len() < 3 {
                help();
                return;
            }
            let key = &args[2];
            match kv.get(key) {
                Some(val) => {
                    println!("{}", val);
                },
                None => {
                    println!("NOT EXISTS {}", key);
                },
            }
        },
        "del" => {
            if args.len() < 3 {
                help();
                return;
            }
            let key = &args[2];
            match kv.del(key) {
                Some(val) => {
                    println!("DELETE {}", val);
                },
                None => {
                    println!("NOT EXISTS {}", key);
                },
            }
        },
        _ => {
            help();
        }
    }
    match kv.commit() {
        Err(why) => { println!("commit error: {}", why.description()) },
        _ => {},
    };
}
