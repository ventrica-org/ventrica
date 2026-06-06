# Contributing to Ventrica

## Requirements

- [Xcode 26.0](https://developer.apple.com/xcode/)
- [Rust 1.96.0](https://rust-lang.org/tools/install/)

```sh
make build
```

## Testing

```sh
cargo build --workspace
# this is the daemon that will do root operations
sudo target/debug/ventricad
# in another tty
cargo run --bin ven -- --help
```