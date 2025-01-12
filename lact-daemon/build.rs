use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=include/");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::builder()
        .header("include/intel.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .dynamic_library_name("IntelDrm")
        .generate_comments(false)
        .generate()
        .expect("Unable to generate intel bindings")
        .write_to_file(out_path.join("intel_bindings.rs"))
        .expect("Couldn't write bindings!");
}
