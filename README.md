# Description
vtail Is a tailing tool just like the standard tail, but with a big advantage. It can tail directories and will pick up new files. It can tail multiple directories. Stop tailing by hitting CTRL+C. The output is sent to standard output so it is pipeable to other processes.

It is written in the language RUST. If following the directions to build it for the MUSL target, it does not require a standard c library.

# Usage
```vtail <path-to-dir1> <path-to-dir2> ...``` - will tail the specified directories

```vtail                                  ``` - will use current directory

# Plans for future developement
Add -d to add directory name to each log line. In version v0.1.1 this is always added.
Add -f to add file name to each log line.

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
