use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    generate_combined_css();
}

fn generate_combined_css() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("combined.css");

    let mut css_files = Vec::new();
    collect_css_files(Path::new("src"), &mut css_files);

    // Sort files to ensure deterministic output
    css_files.sort();

    let mut combined_css = String::new();
    for file in css_files {
        let content = fs::read_to_string(&file).expect("Could not read CSS file");
        combined_css.push_str(&format!("/* Source: {} */\n", file.display()));
        combined_css.push_str(&content);
        combined_css.push('\n');

        println!("cargo:rerun-if-changed={}", file.display());
    }

    fs::write(dest_path, combined_css).expect("Could not write combined CSS file");
}

fn collect_css_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_css_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("css") {
                files.push(path);
            }
        }
    }
}
