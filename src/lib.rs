extern crate nix;

pub mod store;

mod hashmap;
mod file;

use std::fmt;
use std::error;
use std::result;

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
