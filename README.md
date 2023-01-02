# rust2git

Rust interface to [libgit2 C Library ](https://github.com/libgit2/libgit2) following tutorial in [Programming Rust: Fast, Safe Systems Development - Chapter 23 Foreign Functions](https://www.amazon.co.uk/Programming-Rust-Fast-Systems-Development/dp/1492052590)

### libgit2 C Library linking

Extended to include building of libgit2 C Library, bit hacky since it downloads and builds via /libgit2-sys and then linked via the .cargo/config.toml environment variables 

### Run 

`cargo run /path/to/git/library/`