//! Put `prism.dll` beside the built `pa3ds.exe`, where `core::speech` loads it from, so a build
//! of pa3ds can speak without anything installed.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=vendor/prism/prism.dll");
    let source = PathBuf::from("vendor/prism/prism.dll");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    // OUT_DIR is target/<profile>/build/<package>/out; the executables are in target/<profile>.
    if let Some(profile) = out.ancestors().nth(3) {
        let _ = std::fs::copy(&source, profile.join("prism.dll"));
    }
}
