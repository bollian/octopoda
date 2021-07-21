fn main() {
    if let Ok(linker_file) = std::env::var("LINKER_FILE") {
        // sometimes we rely on external crates and their linker files
        println!("cargo:rerun-if-changed={}", linker_file);
    }
    println!("cargo:rerun-if-changed=build.rs");
}
