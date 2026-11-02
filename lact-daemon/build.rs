use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo::rerun-if-changed=include/");

    gen_intel_bindings();

    #[cfg(feature = "nvidia")]
    gen_nvidia_bindings();

    gen_vulkan_constants();

    #[cfg(feature = "schema-generate")]
    gen_api_schema();
}

/// Writes the JSON schemas describing the daemon API into `lact-schema/schema`.
#[cfg(feature = "schema-generate")]
fn gen_api_schema() {
    use lact_schema::{Request, Response};
    use schemars::{JsonSchema, Schema, schema_for};
    use std::{fs::File, io::BufWriter};

    println!("cargo::rerun-if-changed=../lact-schema/src");

    let schema_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../lact-schema/schema");
    fs::create_dir_all(&schema_dir).expect("Couldn't create the schema directory");

    write_schema::<Response>(&schema_dir.join("response.json"));
    write_schema::<Request>(&schema_dir.join("request.json"));

    fn write_schema<T: JsonSchema>(path: &Path) {
        let schema: Schema = schema_for!(T);

        let file = File::create(path).expect("Couldn't create the schema file");
        serde_json::to_writer_pretty(BufWriter::new(file), &schema)
            .expect("Couldn't write the schema file");
    }
}

fn gen_intel_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::builder()
        .header("include/intel.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .dynamic_library_name("IntelDrm")
        // Don't generate bindings for functions and types related to file access
        // This is needed to avoid IntelDrm structure being marked as !Send
        .allowlist_item("(DRM|drm).*")
        .blocklist_item(".*(FILE|file).*")
        .generate_comments(false)
        .generate()
        .expect("Unable to generate intel bindings")
        .write_to_file(out_path.join("intel_bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(feature = "nvidia")]
fn gen_nvidia_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::builder()
        .header("include/nvidia.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate_comments(false)
        .clang_arg("-Iinclude/nvidia/src/common/sdk/nvidia/inc")
        .generate()
        .expect("Unable to generate nvidia bindings")
        .write_to_file(out_path.join("nvidia_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn gen_vulkan_constants() {
    println!("cargo::rerun-if-changed=vulkan_schema.json");
    let out_dir = env::var("OUT_DIR").unwrap();

    let contents = fs::read_to_string("vulkan_schema.json").unwrap();
    let schema: Value = serde_json::from_str(&contents).unwrap();

    let object_properties = schema
        .pointer("/properties/capabilities/additionalProperties/properties")
        .unwrap();

    let extension_names = object_properties
        .pointer("/extensions/properties")
        .unwrap()
        .as_object()
        .expect("Extensions is not an object")
        .keys()
        .collect::<Vec<_>>();

    let feature_names = object_properties
        .pointer("/features/properties")
        .unwrap()
        .as_object()
        .expect("Features is not an object")
        .values()
        .flat_map(|value| {
            let raw_ref = value
                .get("$ref")
                .unwrap_or_else(|| panic!("Could not get ref for vulkan feature '{value}'"));
            let pointer = &raw_ref.as_str().unwrap()[1..];

            let definition = schema.pointer(pointer).unwrap();
            let definition_properties = definition
                .get("properties")
                .unwrap_or_else(|| panic!("Definition at '{pointer}' has no properties"));
            definition_properties.as_object().unwrap().keys()
        })
        .collect::<Vec<_>>();

    let constants = format!(
        r#"
        const VULKAN_EXTENSIONS: [&str; {}] = [{}];
        const VULKAN_FEATURES: [&str; {}] = [{}];
        "#,
        extension_names.len(),
        extension_names
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", "),
        feature_names.len(),
        feature_names
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", "),
    );

    let extensions_file = Path::new(&out_dir).join("vulkan_constants.rs");
    fs::write(extensions_file, constants).unwrap();
}
