use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Set the output path for the openapi.json file
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("openapi.json");

    // Generate the OpenAPI documentation
    let openapi = Mustang::ApiDoc::openapi();
    let json = openapi
        .to_pretty_json()
        .expect("Failed to serialize OpenAPI schema");

    // Write the JSON to the file
    let mut file = File::create(&dest_path).expect("Failed to create openapi.json file");
    file.write_all(json.as_bytes())
        .expect("Failed to write to openapi.json file");

    println!("cargo:rerun-if-changed=src/main.rs");
}
