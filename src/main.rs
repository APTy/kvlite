/// kvlite
extern crate kvlite;

use std::env::args;
use std::error::Error;
use std::process::exit;
use kvlite::store::Store;

/// Default database file name.
const DB_FILENAME: &'static str = "./db.kvlite";

/// Prints CLI usage information.
fn help() -> ! {
    println!("usage: kvlite <command> [<args>]\n");
    println!("kvlite is a key-value store backed by the local file system.\n");
    println!("commands:");
    println!("    set <key> <value>        Create or update a key's value.");
    println!("    get <key>                Look up a key's value.");
    println!("    del <key>                Remove a key.");
    exit(1);
}

fn main() {
    let kv = Store::new(DB_FILENAME);

    let args: Vec<String> = args().collect();
    if args.len() == 1 { help(); }

    let cmd = &args[1];
    let res = match cmd.as_str() {
        "set" => {
            if args.len() < 4 { help(); }
            kv.set(&args[2], &args[3])
        },
        "get" => {
            if args.len() < 3 { help(); }
            kv.get(&args[2])
        },
        "del" => {
            if args.len() < 3 { help(); }
            kv.del(&args[2])
        },
        _ => {
            kv.noop(cmd)
        },
    };
    match res {
        Ok(msg) => { println!("{}", msg.display()); },
        Err(why) => { println!("{}", why.description()); },
    }
}
