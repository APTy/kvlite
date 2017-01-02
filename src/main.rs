/// kvlite
extern crate kvlite;

use std::env::args;
use std::error::Error;
use std::process::exit;
use kvlite::FileHashMap;

/// Default database file name.
const DB_FILENAME: &'static str = "./db.kvlite";

/// Prints CLI usage information.
fn help() -> ! {
    println!("usage: kvl <command> [<args>]\n");
    println!("kvlite is a key-value store backed by the local file system.\n");
    println!("commands:");
    println!("    set <key> <value>        Create or update a key's value.");
    println!("    get <key>                Look up a key's value.");
    println!("    del <key>                Remove a key.");
    exit(1);
}

fn main() {
    let kv = FileHashMap::new(DB_FILENAME);

    let args: Vec<String> = args().collect();
    if args.len() == 1 { help(); }

    let cmd = &args[1];
    match cmd.as_str() {
        "set" => {
            if args.len() < 4 { help(); }
            match kv.insert(&args[2], &args[3]) {
                Err(why) => println!("{}", why.description()),
                Ok(_) => println!("OK"),
            }
        },
        "get" => {
            if args.len() < 3 { help(); }
            match kv.get(&args[2]) {
                Err(why) => println!("{}", why.description()),
                Ok(val) => println!("\"{}\"", val),
            }
        },
        "del" => {
            if args.len() < 3 { help(); }
            match kv.remove(&args[2]) {
                Err(why) => println!("{}", why.description()),
                Ok(_) => println!("OK"),
            }
        },
        _ => { println!("unknown command: {}", cmd) },
    };
}
