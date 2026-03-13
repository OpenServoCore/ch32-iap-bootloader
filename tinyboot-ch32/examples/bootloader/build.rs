fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search={dir}");
    if cfg!(feature = "defmt") {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }
}
