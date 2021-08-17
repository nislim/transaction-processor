# Transaction Processor
Process a transaction file using Rust.

## Dependencies
* For parsing the transaction file [nom](https://crates.io/crates/nom) is used.
* For reading, writing and task / thread management [tokio](https://crates.io/crates/tokio) is used.

## Build
```bash
cargo build
```

Note to ensure optimal performance build with --release

```bash
cargo build --release
```
## Run

```bash
cargo run -- filename.csv
```