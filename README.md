# bitvavo-api

A crate for interacting with the Bitvavo API.

## Documentation

Visit [docs.rs](https://docs.rs/bitvavo-api) for how to use the crate.

## Usage

Make sure to include this line in your dependencies

```toml
bitvavo-api = "0.1"
```

and then use the API

```rust
use bitvavo_api as bitvavo;
bitvavo::time().await.unwrap();
```
