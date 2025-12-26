use std::fs;
use std::path::Path;

fn main() {
    // Temporarily disable export list to see what symbols are generated
    // #[cfg(all(feature = "clap", target_os = "macos"))]
    // {
    //     println!("cargo:rustc-cdylib-link-arg=-Wl,-exported_symbols_list,macos-clap-symbols.txt");
    // }

    // Embed wavetable files at compile time
    println!("cargo:rerun-if-changed=assets/wavetables");

    let wavetable_dir = Path::new("assets/wavetables");
    if wavetable_dir.exists() {
        // Generate embedded wavetable module
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("embedded_wavetables.rs");

        let mut code = String::from("// Auto-generated embedded wavetables\n");
        code.push_str("pub const EMBEDDED_WAVETABLES: &[(&str, &[u8])] = &[\n");

        // Get cargo manifest directory (workspace root)
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        if let Ok(entries) = fs::read_dir(wavetable_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wav") {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        // Build absolute path from manifest directory
                        let absolute_path = Path::new(&manifest_dir).join(&path);
                        let path_str = absolute_path.to_str().unwrap();
                        // Use raw strings and absolute paths
                        code.push_str(&format!(
                            "    (r#\"{}\"#, include_bytes!(r#\"{}\"#) as &[u8]),\n",
                            filename, path_str
                        ));
                    }
                }
            }
        }

        code.push_str("];\n");
        fs::write(dest_path, code).unwrap();
    }
}
