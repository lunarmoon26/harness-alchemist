use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let package_path = manifest_dir.join("package.json");
    println!("cargo:rerun-if-changed={}", package_path.display());

    let package: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&package_path).expect("read package.json"))
            .expect("parse package.json");
    let npm_version = package["version"].as_str().expect("package.json version");
    let cargo_version = env::var("CARGO_PKG_VERSION").expect("Cargo package version");
    assert_eq!(
        npm_version, cargo_version,
        "package.json and Cargo.toml versions must match"
    );
    println!("cargo:rustc-env=HARNESS_ALCHEMIST_VERSION={npm_version}");
    println!(
        "cargo:rustc-env=HARNESS_ALCHEMIST_TARGET={}",
        env::var("TARGET").expect("Cargo target triple")
    );
}
