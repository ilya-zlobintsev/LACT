use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=drm");
    println!("cargo:rustc-link-lib=drm_amdgpu");
    println!("cargo:rustc-link-lib=drm_intel");

    println!("cargo::rerun-if-changed=wrapper/");

    let bindings = bindgen::builder()
        .header("wrapper/intel.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate intel bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("intel_bindings.rs"))
        .expect("Couldn't write bindings!");
}
