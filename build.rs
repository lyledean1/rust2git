use std::env;

fn main() {
    env::set_var("DYLD_LIBRARY_PATH", "/Users/lyledean/rust/rust2git/libgit2-sys/target/source-v1.5.0/build");
    println!(r"DYLD_LIBRARY_PATH=/Users/lyledean/rust/rust2git/libgit2-sys/target/source-v1.5.0/build");
    println!(r"cargo:rustc-link-search=native=/Users/lyledean/rust/rust2git/libgit2-sys/target/source-v1.5.0/build");
}