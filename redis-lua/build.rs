use rustc_version::{version_meta, Channel};

fn main() {
    if version_meta().unwrap().channel != Channel::Stable {
        println!("cargo:rustc-cfg=unstable");
    }
}
