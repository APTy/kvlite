extern crate nix;

pub mod store;

mod hashmap;
mod file;

use std::io;
use std::result;

/// KVLite Result Type
pub type Result<T> = result::Result<T, Error>;

/// KVLite Error Type
pub enum Error {
    IO(io::Error),
}
