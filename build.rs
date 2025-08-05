use std::env;

fn main() {
    // Tell cargo to rerun this script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    // Set linking type for Windows MSVC
    if env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc" {
        println!("cargo:rustc-link-lib=static=vcruntime");
    }

    // Link against protobuf libraries if needed
    println!("cargo:rustc-link-lib=dylib=protobuf");
}
