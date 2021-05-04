fn main() {
    let linker_file = std::env::var("LINKER_FILE").unwrap();

    println!("cargo:rerun-if-changed={}", linker_file);
    println!("cargo:rerun-if-changed=build.rs");
}
