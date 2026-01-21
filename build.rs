use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::warning={:?}", env::var("OUT_DIR"))
}
