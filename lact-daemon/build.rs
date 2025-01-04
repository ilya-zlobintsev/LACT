use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=drm");
    println!("cargo:rustc-link-lib=drm_amdgpu");
    println!("cargo:rustc-link-lib=drm_intel");

    println!("cargo::rerun-if-changed=headers/");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::builder()
        .header("headers/i915.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate intel bindings")
        .write_to_file(out_path.join("i915_bindings.rs"))
        .expect("Couldn't write bindings!");

    // bindgen::builder()
    //     .header("headers/xe.h")
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    //     .generate()
    //     .expect("Unable to generate intel bindings")
    //     .write_to_file(out_path.join("xe_bindings.rs"))
    //     .expect("Couldn't write bindings!");
}
