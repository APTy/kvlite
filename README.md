# kvlite

A key-value store backed by your local file system. Mostly a toy project to learn Rust.

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
kvlite = "0.1.1"
```

Next, add this to your crate:

```rust
extern crate kvlite;

use kvlite::Store;
```

Now you're ready to use kvlite!
