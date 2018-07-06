# Building for MUSL
```
cargo build --release --target=x86_64-unknown-linux-musl
cd target/x86_64-unknown-linux-musl/release
strip vtail
```

# RUSTUP
platform manager: https://blog.rust-lang.org/2016/05/13/rustup.html

Adding target: `rustup target add x86_64-unknown-linux-musl`

Showing targets: `rustup show`
