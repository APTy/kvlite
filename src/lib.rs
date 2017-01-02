//! A key-value store backed by your local file system.
//!
//! The underlying hashmap uses an inefficient hashing algorithm to
//! place keys and values into large and unoptimized buckets that it
//! reads/writes from a file.
//!
//! The file size is very large, and does not re-allocate memory.
//! Also, the hashmap doesn't resize its keyspace. I may or may not
//! fix these things.. this project was mostly just to learn Rust.
//!
//! Reads are something like 2k QPS, writes 500 QPS. It's thread- and
//! process-safe (probably).
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! kvlite = "0.1.2"
//! ```
//!
//! ## Examples
//!
//!
//! ```
//! extern crate kvlite;
//!
//! use kvlite::FileHashMap;
//!
//! let kv = FileHashMap::new("myfile.kvlite");
//!
//! kv.insert("foo", "bar");
//!
//! let foo = kv.get("foo").unwrap();
//!
//! println!("foo: {}", foo);  // prints: "foo: bar"
//! ```

extern crate nix;

pub mod hashmap;
mod file;

use std::fmt;
use std::error;
use std::result;

pub use hashmap::FileHashMap;

/// KVLite Result Type
pub type Result<T> = result::Result<T, Error>;

/// KVLite Error Type
#[derive(Debug, PartialEq)]
pub enum Error {
    IO,
    NotFound,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::IO => "i/o error",
            &Error::NotFound => "key not found",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::IO => write!(f, "io error"),
            &Error::NotFound => write!(f, "not found error"),
        }
    }
}
