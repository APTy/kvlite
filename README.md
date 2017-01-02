# kvlite

A key-value store backed by your local file system. Mostly a toy project to learn Rust.

[Documentation](https://docs.rs/kvlite)

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
kvlite = "0.1.2"
```

Next, add this to your crate:

```rust
extern crate kvlite;

use kvlite::Store;
```

See documentation for library usage.


## CLI Usage

With [rust](https://www.rustup.rs/) installed, you can use `cargo install kvlite` to get the `kvl` CLI tool.

```
usage: kvl <command> [<args>]

kvlite is a key-value store backed by the local file system.

commands:
    set <key> <value>        Create or update a key's value.
    get <key>                Look up a key's value.
    del <key>                Remove a key.
```
