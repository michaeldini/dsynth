fn main() {
    // Temporarily disable export list to see what symbols are generated
    // #[cfg(all(feature = "clap", target_os = "macos"))]
    // {
    //     println!("cargo:rustc-cdylib-link-arg=-Wl,-exported_symbols_list,macos-clap-symbols.txt");
    // }
}
