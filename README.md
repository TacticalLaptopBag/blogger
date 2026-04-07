# Blogger

## Developer Setup

Install `diesel_cli` using either [cargo-binstall] or `cargo install`:
```bash
cargo install binstall
cargo binstall diesel_cli
```
```bash
cargo install diesel_cli --no-default-features --features sqlite
```

Apply migrations:
```bash
diesel migration run
```


[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
