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

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.