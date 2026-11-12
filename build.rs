fn main() {
    println!("cargo:rerun-if-changed=config.default.toml");
    println!("cargo:rerun-if-changed=build.rs");
}
